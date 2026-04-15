//! Defines the Assign action for storing values during benchmark execution.
//!
//! # Examples
//!
//! ```rust
//! use floodr::actions::assign::Assign;
//! use serde_yaml;
//! 
//! let plan_data = r#"
//! name: Assign a key
//! assign:
//!   key: foo
//!   value: 200"#;
//! 
//! let action_data = serde_yaml::from_str(plan_data).expect("Failed to parse");
//! println!("{}", Assign::is_that_you(&action_data)); // true
//! 
//! let s = Assign::new(&action_data, None);
//! ```

use async_trait::async_trait;
use colored::*;
use serde_json::json;
use serde_yaml::Value;

use crate::actions::Runnable;
use crate::actions::extract;
use crate::engine::benchmark::{Context, Pool, Reports};
use crate::parsing::config::Config;

/// Represents an assignment action to store variables in the context.
///
/// # Fields
///
/// - `name` (`String`) - The name of the assign action (will show up in CLI)
/// - `key` (`String`) - The field to store the value in
/// - `value` (`String`) - The value to store
///
/// # Examples
///
/// ```rust
/// use floodr::actions::assign::Assign;
/// use serde_yaml;
/// 
/// let plan_data = r#"
/// name: Assign a key
/// assign:
///   key: foo
///   value: 200"#;
/// 
/// let action_data = serde_yaml::from_str(plan_data).expect("Failed to parse");
/// println!("{}", Assign::is_that_you(&action_data)); // true
/// 
/// let s = Assign::new(&action_data, None);
/// ```
#[derive(Clone)]
pub struct Assign {
  name: String,
  key: String,
  value: String,
}

impl Assign {
  /// Checks if the provided YAML item represents an `Assign` action.
  ///
  /// # Arguments
  ///
  /// - `item` (`&Value`) - The YAML item
  ///
  /// # Returns
  ///
  /// - `bool` - True if the item provided is an assign
  ///
  /// # Examples
  ///
  /// ```rust
  /// use floodr::actions::assign::Assign;
  /// use serde_yaml;
  /// 
  /// let plan_data = r#"
  /// name: Assign a key
  /// assign:
  ///   key: foo
  ///   value: 200"#;
  /// 
  /// let action_data = serde_yaml::from_str(plan_data).expect("Failed to parse");
  /// 
  /// println!("{}", Assign::is_that_you(&action_data)); // true
  /// ```
  pub fn is_that_you(item: &Value) -> bool {
    item.get("assign").and_then(|v| v.as_mapping()).is_some()
  }

  /// Creates a new `Assign` action from a YAML item.
  ///
  /// # Arguments
  ///
  /// - `item` (`&Value`) - The YAML item
  /// - `_with_item` (`Option<Value>`) - The item to use for the request
  ///
  /// # Returns
  ///
  /// - `Assign` - The new `Assign` action
  ///
  /// # Examples
  ///
  /// ```rust
  /// use floodr::actions::assign::Assign;
  /// use serde_yaml;
  /// 
  /// let plan_data = r#"
  /// name: Assign a key
  /// assign:
  ///   key: foo
  ///   value: 200"#;
  /// 
  /// let action_data = serde_yaml::from_str(plan_data).expect("Failed to parse");
  /// 
  /// let s = Assign::new(&action_data, None);
  /// ```
  pub fn new(item: &Value, _with_item: Option<Value>) -> Assign {
    let name = extract(item, "name");
    let assign_val = item.get("assign").expect("assign field is required");
    let key = extract(assign_val, "key");
    let value = extract(assign_val, "value");

    Assign {
      name,
      key,
      value,
    }
  }
}

#[async_trait]
impl Runnable for Assign {
  async fn execute(&self, context: &mut Context, _reports: &mut Reports, _pool: &Pool, config: &Config) {
    if !config.quiet {
      println!("{:width$} {}={}", self.name.green(), self.key.cyan().bold(), self.value.magenta(), width = 25);
    }

    context.insert(self.key.to_owned(), json!(self.value.to_owned()));
  }
}
