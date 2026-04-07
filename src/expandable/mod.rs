//! Provides the code for handling expandable directives (things that expand into multiple actions) such as iteration picking and random shuffling.
//!
//! When using a benchmark file, you can use the following expandable directives:
//! - `include` - Includes another benchmark file
//! - `with_items` - Dispatches multiple requests based on a literal list
//! - `with_items_range` - Dispatches multiple requests based on a numeric range
//! - `with_items_from_csv` - Dispatches multiple requests based on a CSV file
//! - `with_items_from_file` - Dispatches multiple requests based on a text file
//!
//! Which correspond to the modules in this directory respectively.
//!
//! # Examples
//!
//! The below YAML
//!
//! ```yaml
//! plan:
//!   - name: Fetch account
//!     request:
//!       url: /api/{{ item }}
//!     with_items:
//!       - 1
//!       - 2
//!       - 3
//! ```
//!
//! Would result in three `Request` actions being created and added to the benchmark.

pub mod include;

mod multi_csv_request;
mod multi_file_request;
mod multi_iter_request;
mod multi_request;

use serde_yaml::Value;

/// Determines the number of items to pick from a list based on an optional 'pick' attribute.
///
/// # Arguments
///
/// - `item` (`&Value`) - The YAML item containing the 'pick' attribute
/// - `with_items` (`&[Value]`) - The list of items to pick from
///
/// # Returns
///
/// - `usize` - The number of items to pick
///
/// # Panics
///
/// - Panics if 'pick' is negative
/// - Panics if 'pick' is greater than the number of available items
///
/// # Examples
///
/// ```rust,ignore
/// use serde_yaml::Value;
/// use floodr::expandable::pick;
///
/// let item = serde_yaml::from_str("pick: 2").unwrap();
/// let with_items = vec![Value::from(1), Value::from(2), Value::from(3)];
/// let n = pick(&item, &with_items);
/// assert_eq!(n, 2);
/// ```
pub fn pick(item: &Value, with_items: &[Value]) -> usize {
  match item.get("pick").and_then(|v| v.as_i64()) {
    Some(value) => {
      if value.is_negative() {
        panic!("pick option should not be negative, but was {value}");
      } else if value as usize > with_items.len() {
        panic!("pick option should not be greater than the provided items, but was {value}");
      } else {
        value as usize
      }
    }
    None => with_items.len(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod pick {
    use super::*;

    #[test]
    fn should_return_the_configured_value() {
      let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\npick: 2\nwith_items:\n  - 1\n  - 2\n  - 3";
      let item = &crate::parsing::reader::read_file_as_yml_from_str(text)[0];
      let pick = pick(item, item.get("with_items").and_then(|v| v.as_sequence()).unwrap());

      assert_eq!(pick, 2);
    }

    #[test]
    fn should_return_the_with_items_length_if_unset() {
      let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\nwith_items:\n  - 1\n  - 2\n  - 3";
      let item = &crate::parsing::reader::read_file_as_yml_from_str(text)[0];
      let pick = pick(item, item.get("with_items").and_then(|v| v.as_sequence()).unwrap());

      assert_eq!(pick, 3);
    }

    #[test]
    #[should_panic(expected = "pick option should not be negative, but was -1")]
    fn should_panic_for_negative_values() {
      let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\npick: -1\nwith_items:\n  - 1\n  - 2\n  - 3";
      let item = &crate::parsing::reader::read_file_as_yml_from_str(text)[0];
      pick(item, item.get("with_items").and_then(|v| v.as_sequence()).unwrap());
    }

    #[test]
    #[should_panic(expected = "pick option should not be greater than the provided items, but was 4")]
    fn should_panic_for_values_greater_than_the_items_list() {
      let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\npick: 4\nwith_items:\n  - 1\n  - 2\n  - 3";
      let item = &crate::parsing::reader::read_file_as_yml_from_str(text)[0];
      pick(item, item.get("with_items").and_then(|v| v.as_sequence()).unwrap());
    }
  }
}
