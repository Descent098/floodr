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
//!   - name: Waiting some seconds
//!     delay:
//!       seconds: 3
//! ```

use async_trait::async_trait;
use colored::*;
use serde_yaml::Value;
use tokio::time::sleep;

use crate::actions::Runnable;
use crate::actions::extract;
use crate::benchmark::{Context, Pool, Reports};
use crate::config::Config;

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
///   - name: Delay for 5 seconds
///     delay:
///       seconds: 5
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
///     seconds: u64,
/// }
///
/// #[derive(Serialize)]
/// struct DelayItem {
///     name: String,
///     delay: DelayItemDetails,
/// }
///
/// let config = DelayItem {
///     name: "Delay for 5 seconds".to_string(),
///     delay: DelayItemDetails {
///         seconds: 5,
///     },
/// };
/// let value = serde_yaml::to_value(config).unwrap();
/// let s = Delay::new(&value, None);
/// ```
#[derive(Clone)]
pub struct Delay {
  name: String, // The name of the delay action (will show up in CLI)
  seconds: u64, // The number of seconds to delay
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
  ///     seconds: u64,
  /// }
  ///
  /// #[derive(Serialize)]
  /// struct DelayItem {
  ///     name: String,
  ///     delay: DelayItemDetails,
  /// }
  ///
  /// let config = DelayItem {
  ///     name: "Delay for 5 seconds".to_string(),
  ///     delay: DelayItemDetails {
  ///         seconds: 5,
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
  ///     seconds: u64,
  /// }
  ///
  /// #[derive(Serialize)]
  /// struct DelayItem {
  ///     name: String,
  ///     delay: DelayItemDetails,
  /// }
  ///
  /// let config = DelayItem {
  ///     name: "Delay for 5 seconds".to_string(),
  ///     delay: DelayItemDetails {
  ///         seconds: 5,
  ///     },
  /// };
  /// let value = serde_yaml::to_value(config).unwrap();
  /// let s = Delay::new(&value, None);
  /// ```
  pub fn new(item: &Value, _with_item: Option<Value>) -> Delay {
    let name = extract(item, "name");
    let delay_val = item.get("delay").expect("delay field is required");
    let seconds = u64::try_from(delay_val.get("seconds").and_then(|v| v.as_i64()).expect("Invalid number of seconds")).expect("Invalid number of seconds");

    Delay {
      name,
      seconds,
    }
  }
}

#[async_trait]
impl Runnable for Delay {
  async fn execute(&self, _context: &mut Context, _reports: &mut Reports, _pool: &Pool, config: &Config) {
    sleep(Duration::from_secs(self.seconds)).await;

    if !config.quiet {
      println!("{:width$} {}{}", self.name.green(), self.seconds.to_string().cyan().bold(), "s".magenta(), width = 25);
    }
  }
}
