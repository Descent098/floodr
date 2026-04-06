//! Defines the Assert action for checking response values against expected values
//!
//! # Examples
//!
//! ```yaml
//! plan:
//!   - name: Fetch account
//!     request:
//!       url: /api/account
//!     assign: foo
//!   - name: Assert request response code
//!     assert: # This part
//!       key: foo.status
//!       value: 200
//! ```

use async_trait::async_trait;
use colored::*;
use serde_json::json;
use serde_yaml::Value;

use crate::actions::Runnable;
use crate::actions::extract;
use crate::benchmark::{Context, Pool, Reports};
use crate::config::Config;
use crate::interpolator;

/// Represents an assertion action in a benchmark plan. Used to verify that variables or responses match expected values.
///
/// # Fields
///
/// - `name` (`String`) - The name of the assert action (whill show up in CLI)
/// - `key` (`String`) - The field to assert against
/// - `value` (`String`) - The expected value of the field
///
/// # Examples
///
/// With a yaml file like:
///
/// ```yaml
/// plan:
///   - name: Fetch account
///     request:
///       url: /api/account
///     assign: foo
///   - name: Assert request response code
///     assert:
///       key: foo.status
///       value: 200
/// ```
///
/// This equates to something like:
///
/// ```
/// use serde::Serialize;
/// use floodr::actions::Assert;
///
/// #[derive(Serialize)]
/// struct AssertItemDetails {
///     key: String,
///     value: String,
/// }
///
/// #[derive(Serialize)]
/// struct AssertItem {
///     name: String,
///     assert: AssertItemDetails,
/// }
///
/// let config = AssertItem {
///     name: "Assert request response code".to_string(),
///     assert: AssertItemDetails {
///         key: "foo.status".to_string(),
///         value: "200".to_string(),
///     },
/// };
/// let value = serde_yaml::to_value(config).unwrap();
/// let s = Assert::new(&value, None);
/// ```
#[derive(Clone)]
pub struct Assert {
  name: String,
  key: String,
  value: String,
}

impl Assert {
  /// Checks if the provided YAML item represents an `Assert` action
  ///
  /// # Arguments
  ///
  /// - `item` (`&Value`) - The YAML item
  ///
  /// # Returns
  ///
  /// - `bool` - True if the item provided is an assert
  ///
  /// # Examples
  ///
  /// ```
  /// use serde::Serialize;
  /// use floodr::actions::Assert;
  ///
  /// #[derive(Serialize)]
  /// struct AssertItemDetails {
  ///     key: String,
  ///     value: String,
  /// }
  ///
  /// #[derive(Serialize)]
  /// struct AssertItem {
  ///     name: String,
  ///     assert: AssertItemDetails,
  /// }
  ///
  /// let config = AssertItem {
  ///     name: "Assert request response code".to_string(),
  ///     assert: AssertItemDetails {
  ///         key: "foo.status".to_string(),
  ///         value: "200".to_string(),
  ///     },
  /// };
  /// let value = serde_yaml::to_value(config).unwrap();
  /// let s = Assert::is_that_you(&value);
  /// ```
  pub fn is_that_you(item: &Value) -> bool {
    item.get("assert").and_then(|v| v.as_mapping()).is_some()
  }

  /// Creates a new `Assert` action from a YAML item
  ///
  /// # Arguments
  ///
  /// - `item` (`&Value`) - The YAML item
  /// - `_with_item` (`Option<Value>`) - The item to use for the request
  ///
  /// # Returns
  ///
  /// - `Assert` - The new `Assert` action
  ///
  /// # Examples
  ///
  /// ```
  /// use serde::Serialize;
  /// use floodr::actions::Assert;
  ///
  /// #[derive(Serialize)]
  /// struct AssertItemDetails {
  ///     key: String,
  ///     value: String,
  /// }
  ///
  /// #[derive(Serialize)]
  /// struct AssertItem {
  ///     name: String,
  ///     assert: AssertItemDetails,
  /// }
  ///
  /// let config = AssertItem {
  ///     name: "Assert request response code".to_string(),
  ///     assert: AssertItemDetails {
  ///         key: "foo.status".to_string(),
  ///         value: "200".to_string(),
  ///     },
  /// };
  /// let value = serde_yaml::to_value(config).unwrap();
  /// let s = Assert::new(&value, None);
  /// ```
  pub fn new(item: &Value, _with_item: Option<Value>) -> Assert {
    let name = extract(item, "name");
    let assert_val = item.get("assert").expect("assert field is required");
    let key = extract(assert_val, "key");
    let value = extract(assert_val, "value");

    Assert {
      name,
      key,
      value,
    }
  }
}

#[async_trait]
impl Runnable for Assert {
  async fn execute(&self, context: &mut Context, _reports: &mut Reports, _pool: &Pool, config: &Config) {
    if !config.quiet {
      println!("{:width$} {}={}?", self.name.green(), self.key.cyan().bold(), self.value.magenta(), width = 25);
    }

    let interpolator = interpolator::Interpolator::new(context);
    let eval = format!("{{{{ {} }}}}", &self.key);
    let stored = interpolator.resolve(&eval, true);
    let assertion = json!(self.value.to_owned());

    if !stored.eq(&assertion) {
      let msg = format!("{}{}{}", "Assertion failed for action ".red(), format!("\"{}\"", self.name).cyan().bold(), format!(": {} != {}", stored, assertion).red());
      print!("{}", msg);

      panic!("{}", msg); // TODO: Should this panic? or should the app just exit with an error code?
    }
  }
}
