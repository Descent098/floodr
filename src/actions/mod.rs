//! Module containing code to help process and execute the various action types
//!
//! When using a benchmark file, you can use the following actions:
//! - `assert` - Asserts that a value is equal to another value
//! - `assign` - Assigns a value to a variable
//! - `delay` - Delays the execution of the next action
//! - `exec` - Executes a command
//! - `request` - Makes a request to a URL
//!
//! Which correspond to the available structs in this module respectively
//!
//! # Implementation Notes
//! - Each action is parsed from a YAML item
//! - Each item is minimally expected to have a `name` field which is used to identify the action. The `name` field is also used to identify the action in the reports.
//! - Each action struct uses `is_that_you` to determine if it should be used for a given YAML item
//! - Each action struct uses `new` to create a new instance of the action from a YAML item
//! - Each action struct implements `Runnable` to provide the core behavior of the action
//!
//! # Examples
//!
//! The below YAML
//!
//! ```yaml
//! plan:
//!   - name: Fetch account
//!     request:
//!       url: /api/account
//!     assign: foo
//!   - name: Assert request response code
//!     assert:
//!       key: foo.status
//!       value: 200
//! ```
//!
//! Would result in a struct for the corresponding action being created and added to the `floodr::benchmark::Context` struct. In this case a `Request` struct and an `Assert` struct would be created and added to the `Context` struct.
//!
//! ```rust,ignore
//! use serde::Serialize;
//! use floodr::actions::{Assert, Request, Runnable};
//!
//! #[derive(Serialize)]
//! struct RequestItemDetails {
//!     url: String,
//! }
//!
//! #[derive(Serialize)]
//! struct RequestItem {
//!     name: String,
//!     request: RequestItemDetails,
//! }
//!
//! let req_item = RequestItem {
//!     name: "Fetch account".to_string(),
//!     request: RequestItemDetails {
//!         url: "/api/account".to_string(),
//!     },
//! };
//! let request_value = serde_yaml::to_value(req_item).unwrap();
//! let request = Request::new(&request_value, None, None);
//! request.execute(
//!     &mut context:floodr::benchmark::Context,
//!     &mut reports:floodr::benchmark::Reports,
//!     &pool:floodr::benchmark::Pool,
//!     &config:floodr::config::Config
//! ).await;
//!
//! #[derive(Serialize)]
//! struct AssertItemDetails {
//!     key: String,
//!     value: String,
//! }
//!
//! #[derive(Serialize)]
//! struct AssertItem {
//!     name: String,
//!     assert: AssertItemDetails,
//! }
//!
//! let assert_item = AssertItem {
//!     name: "Assert request response code".to_string(),
//!     assert: AssertItemDetails {
//!         key: "foo.status".to_string(),
//!         value: "200".to_string(),
//!     },
//! };
//! let value = serde_yaml::to_value(assert_item).unwrap();
//!
//! let action = Assert::new(&value, None);
//! action.execute(
//!     &mut context:floodr::benchmark::Context,
//!     &mut reports:floodr::benchmark::Reports,
//!     &pool:floodr::benchmark::Pool,
//!     &config:floodr::config::Config
//! ).await;
//! ```
use async_trait::async_trait;
use serde_yaml::Value;

mod assert;
mod assign;
mod delay;
mod exec;
mod request;

pub use self::assert::Assert;
pub use self::assign::Assign;
pub use self::delay::Delay;
pub use self::exec::Exec;
pub use self::request::Request;
pub use self::request::Body;

use crate::engine::benchmark::{Context, Pool, Reports};
use crate::parsing::config::Config;

use std::fmt;

/// Defines the core behavior expected from any runnable action in a plan
///
/// # Notes
///
/// - Implemented by each action type and is run for each iteration in the benchmark
#[async_trait]
pub trait Runnable {
  /// Executes the action given the current context, state, and configs.
  async fn execute(&self, context: &mut Context, reports: &mut Reports, pool: &Pool, config: &Config);
}

/// Represents the result and statistics of an executed action.
///
/// # Fields
///
/// - `name` (`String`) - The name of the action
/// - `duration` (`f64`) - The duration of the action in seconds
/// - `status` (`u16`) - The status code of the action
///
/// # Examples
///
/// ```rust
/// use floodr::actions::Report;
///
/// let report = Report {
///     name: "Fetch account".to_string(),
///     duration: 1.0,
///     status: 200,
/// };
/// ```
#[derive(Clone)]
pub struct Report {
  pub name: String,
  pub duration: f64,
  pub status: u16,
}

impl fmt::Debug for Report {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "\n- name: {}\n  duration: {}\n", self.name, self.duration)
  }
}

impl fmt::Display for Report {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "\n- name: {}\n  duration: {}\n  status: {}\n", self.name, self.duration, self.status)
  }
}

/// Extracts an optional string value from a YAML node
///
/// # Arguments
///
/// - `item` (`&Value`) - The YAML item
/// - `attr` (`&str`) - The attribute to extract
///
/// # Returns
///
/// - `Option<String>` - The extracted value if found
///
/// # Notes
///
/// - If the value is a mapping, it will panic
/// - If the value is not found, it will return None
///
/// # Examples
///
/// ```rust
/// use floodr::actions::extract_optional;
/// use serde_yaml::Value;
///
/// let item = serde_yaml::from_str("name: Fetch account").unwrap();
/// let value = extract_optional(&item, "name");
/// assert_eq!(value, Some("Fetch account".to_string()));
/// ```
pub fn extract_optional<'a>(item: &'a Value, attr: &'a str) -> Option<String> {
  if let Some(s) = item.get(attr).and_then(|v| v.as_str()) {
    Some(s.to_string())
  } else if item.get(attr).and_then(|v| v.as_mapping()).is_some() {
    panic!("`{attr}` needs to be a string. Try adding quotes");
  } else {
    None
  }
}

/// Extracts a required string value from a YAML node
///
/// # Arguments
///
/// - `item` (`&Value`) - The YAML item
/// - `attr` (`&str`) - The attribute to extract
///
/// # Returns
///
/// - `String` - The extracted value
///
/// # Notes
///
/// - If the value is a mapping, it will panic
/// - If the value is not found, it will panic
///
/// # Examples
///
/// ```rust
/// use floodr::actions::extract;
/// use serde_yaml::Value;
///
/// let item = serde_yaml::from_str("name: Fetch account").unwrap();
/// let value = extract(&item, "name");
/// assert_eq!(value, "Fetch account".to_string());
/// ```
pub fn extract<'a>(item: &'a Value, attr: &'a str) -> String {
  if let Some(s) = item.get(attr).and_then(|v| v.as_i64()) {
    s.to_string()
  } else if let Some(s) = item.get(attr).and_then(|v| v.as_str()) {
    s.to_string()
  } else if item.get(attr).and_then(|v| v.as_mapping()).is_some() {
    panic!("`{attr}` is required needs to be a string. Try adding quotes");
  } else {
    panic!("Unknown node `{}` => {:?}", attr, item.get(attr));
  }
}
