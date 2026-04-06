//! Defines the Exec action for running arbitrary shell commands.
//!
//! # Examples
//!
//! ```yaml
//! plan:
//!   - name: Run a command
//!     exec:
//!       command: echo "Hello, world!"
//!     assign: my_output
//! ```

use async_trait::async_trait;
use colored::*;
use serde_json::json;
use serde_yaml::Value;
use std::process::Command;

use crate::actions::Runnable;
use crate::actions::{extract, extract_optional};
use crate::benchmark::{Context, Pool, Reports};
use crate::config::Config;
use crate::interpolator;

/// Represents an execution action to run external commands during a benchmark.
///
/// # Fields
///
/// - `assign` (`Option<String>`) - The variable to assign the command output to
///
/// # Examples
///
/// With a yaml file like:
///
/// ```yaml
/// plan:
///   - name: Run a command
///     exec:
///       command: echo "Hello, world!"
///     assign: my_output
/// ```
///
/// This equates to something like:
///
/// ```
/// use serde::Serialize;
/// use floodr::actions::Exec;
///
/// #[derive(Serialize)]
/// struct ExecItemDetails {
///     command: String,
/// }
///
/// #[derive(Serialize)]
/// struct ExecItem {
///     name: String,
///     exec: ExecItemDetails,
///     assign: Option<String>,
/// }
///
/// let config = ExecItem {
///     name: "Run a command".to_string(),
///     exec: ExecItemDetails {
///         command: "echo \"Hello, world!\"".to_string(),
///     },
///     assign: Some("my_output".to_string()),
/// };
/// let value = serde_yaml::to_value(config).unwrap();
/// let s = Exec::new(&value, None);
/// ```
#[derive(Clone)]
pub struct Exec {
  name: String, // The name of the exec action (will show up in CLI)
  command: String, // The command to execute
  pub assign: Option<String>,
}

impl Exec {
  /// Checks if the provided YAML item represents an `Exec` action.
  ///
  /// # Arguments
  ///
  /// - `item` (`&Value`) - The YAML item
  ///
  /// # Returns
  ///
  /// - `bool` - True if the item provided is an exec
  ///
  /// # Examples
  ///
  /// ```
  /// use serde::Serialize;
  /// use floodr::actions::Exec;
  ///
  /// #[derive(Serialize)]
  /// struct ExecItemDetails {
  ///     command: String,
  /// }
  ///
  /// #[derive(Serialize)]
  /// struct ExecItem {
  ///     name: String,
  ///     exec: ExecItemDetails,
  ///     assign: Option<String>,
  /// }
  ///
  /// let config = ExecItem {
  ///     name: "Run a command".to_string(),
  ///     exec: ExecItemDetails {
  ///         command: "echo \"Hello, world!\"".to_string(),
  ///     },
  ///     assign: Some("my_output".to_string()),
  /// };
  /// let value = serde_yaml::to_value(config).unwrap();
  /// let s = Exec::is_that_you(&value);
  /// ```
  pub fn is_that_you(item: &Value) -> bool {
    item.get("exec").and_then(|v| v.as_mapping()).is_some()
  }

  /// Creates a new `Exec` action from a YAML item.
  ///
  /// # Arguments
  ///
  /// - `item` (`&Value`) - The YAML item
  /// - `_with_item` (`Option<Value>`) - The item to use for the request
  ///
  /// # Returns
  ///
  /// - `Exec` - The new `Exec` action
  ///
  /// # Examples
  ///
  /// ```
  /// use serde::Serialize;
  /// use floodr::actions::Exec;
  ///
  /// #[derive(Serialize)]
  /// struct ExecItemDetails {
  ///     command: String,
  /// }
  ///
  /// #[derive(Serialize)]
  /// struct ExecItem {
  ///     name: String,
  ///     exec: ExecItemDetails,
  ///     assign: Option<String>,
  /// }
  ///
  /// let config = ExecItem {
  ///     name: "Run a command".to_string(),
  ///     exec: ExecItemDetails {
  ///         command: "echo \"Hello, world!\"".to_string(),
  ///     },
  ///     assign: Some("my_output".to_string()),
  /// };
  /// let value = serde_yaml::to_value(config).unwrap();
  /// let s = Exec::new(&value, None);
  /// ```
  pub fn new(item: &Value, _with_item: Option<Value>) -> Exec {
    let name = extract(item, "name");
    let exec_val = item.get("exec").expect("exec field is required");
    let command = extract(exec_val, "command");
    let assign = extract_optional(item, "assign");

    Exec {
      name,
      command,
      assign,
    }
  }
}

#[async_trait]
impl Runnable for Exec {
  async fn execute(&self, context: &mut Context, _reports: &mut Reports, _pool: &Pool, config: &Config) {
    if !config.quiet {
      println!("{:width$} {}", self.name.green(), self.command.cyan().bold(), width = 25);
    }

    let final_command = interpolator::Interpolator::new(context).resolve(&self.command, !config.relaxed_interpolations);

    let args = ["bash", "-c", "--", final_command.as_str()];

    let execution = Command::new(args[0]).args(&args[1..]).output().expect("Couldn't run it");

    let output: String = String::from_utf8_lossy(&execution.stdout).into();
    let output = output.trim_end().to_string();

    if let Some(ref key) = self.assign {
      context.insert(key.to_owned(), json!(output));
    }
  }
}
