//! Defines the Delay action for introducing pauses between benchmark executions.
//!
//! # Examples
//!
//! ```yaml
//! plan:
//!   - name: Fetch users
//!     request:
//!       url: /api/users.json
//! 
//!   - name: Waiting some milliseconds
//!     delay:
//!       milliseconds: 3000
//! ```
//!
//! # Fallback Behavior
//!
//! - If `milliseconds` is specified, it is used directly.
//! - If `seconds` is specified, it is automatically converted to milliseconds.
//! - If both are specified, `milliseconds` takes precedence.

use async_trait::async_trait;
use colored::*;
use serde_yaml::Value;
use tokio::time::sleep;

use crate::actions::Runnable;
use crate::actions::extract;
use crate::engine::benchmark::{Context, Pool, Reports};
use crate::parsing::config::Config;

use std::convert::TryFrom;
use std::time::Duration;

/// Represents a delay action in a benchmark plan.
///
/// # Examples
///
/// With a yaml file like:
///
/// ```yaml
/// plan:
///   - name: Delay for 500ms
///     delay:
///       milliseconds: 500
/// ```
///
/// This equates to something like:
///
/// ```
/// use serde::Serialize;
/// use floodr::actions::Delay;
///
/// #[derive(Serialize)]
/// struct DelayItemDetails {
///     milliseconds: u64,
/// }
///
/// #[derive(Serialize)]
/// struct DelayItem {
///     name: String,
///     delay: DelayItemDetails,
/// }
///
/// let config = DelayItem {
///     name: "Delay for 500ms".to_string(),
///     delay: DelayItemDetails {
///         milliseconds: 500,
///     },
/// };
/// let value = serde_yaml::to_value(config).unwrap();
/// let s = Delay::new(&value, None);
/// ```
#[derive(Clone)]
pub struct Delay {
  name: String,      // The name of the delay action (will show up in CLI)
  milliseconds: u64, // The number of milliseconds to delay
}

impl Delay {
  /// Checks if the provided YAML item represents a `Delay` action.
  ///
  /// # Arguments
  ///
  /// - `item` (`&Value`) - The YAML item
  ///
  /// # Returns
  ///
  /// - `bool` - True if the item provided is a delay
  ///
  /// # Examples
  ///
  /// ```
  /// use serde::Serialize;
  /// use floodr::actions::Delay;
  ///
  /// #[derive(Serialize)]
  /// struct DelayItemDetails {
  ///     milliseconds: u64,
  /// }
  ///
  /// #[derive(Serialize)]
  /// struct DelayItem {
  ///     name: String,
  ///     delay: DelayItemDetails,
  /// }
  ///
  /// let config = DelayItem {
  ///     name: "Delay for 500ms".to_string(),
  ///     delay: DelayItemDetails {
  ///         milliseconds: 500,
  ///     },
  /// };
  /// let value = serde_yaml::to_value(config).unwrap();
  /// let s = Delay::is_that_you(&value);
  /// ```
  pub fn is_that_you(item: &Value) -> bool {
    item.get("delay").and_then(|v| v.as_mapping()).is_some()
  }

  /// Creates a new `Delay` action from a YAML item.
  ///
  /// Supports both `milliseconds` and `seconds` (converted to ms) fields.
  /// `milliseconds` takes precedence if both are provided.
  ///
  /// # Arguments
  ///
  /// - `item` (`&Value`) - The YAML item
  /// - `_with_item` (`Option<Value>`) - The item to use for the request
  ///
  /// # Returns
  ///
  /// - `Delay` - The new `Delay` action
  ///
  /// # Examples
  ///
  /// ```
  /// use serde::Serialize;
  /// use floodr::actions::Delay;
  ///
  /// #[derive(Serialize)]
  /// struct DelayItemDetails {
  ///     milliseconds: u64,
  /// }
  ///
  /// #[derive(Serialize)]
  /// struct DelayItem {
  ///     name: String,
  ///     delay: DelayItemDetails,
  /// }
  ///
  /// let config = DelayItem {
  ///     name: "Delay for 500ms".to_string(),
  ///     delay: DelayItemDetails {
  ///         milliseconds: 500,
  ///     },
  /// };
  /// let value = serde_yaml::to_value(config).unwrap();
  /// let s = Delay::new(&value, None);
  /// ```
  pub fn new(item: &Value, _with_item: Option<Value>) -> Delay {
    let name = extract(item, "name");
    let delay_val = item.get("delay").expect("delay field is required");
    
    let milliseconds = if let Some(ms) = delay_val.get("milliseconds").and_then(|v| v.as_i64()) {
      u64::try_from(ms).expect("Invalid number of milliseconds")
    } else if let Some(s) = delay_val.get("seconds").and_then(|v| v.as_i64()) {
      u64::try_from(s).expect("Invalid number of seconds") * 1000
    } else {
      panic!("Either 'seconds' or 'milliseconds' must be provided in delay action");
    };

    Delay {
      name,
      milliseconds,
    }
  }
}

#[async_trait]
impl Runnable for Delay {
  async fn execute(&self, _context: &mut Context, _reports: &mut Reports, _pool: &Pool, config: &Config) {
    sleep(Duration::from_millis(self.milliseconds)).await;

    if !config.quiet {
      if self.milliseconds % 1000 == 0 {
        println!("{:width$} {}{}", self.name.green(), (self.milliseconds / 1000).to_string().cyan().bold(), "s".magenta(), width = 25);
      } else {
        println!("{:width$} {}{}", self.name.green(), self.milliseconds.to_string().cyan().bold(), "ms".magenta(), width = 25);
      }
    }
  }
}
