//! CSV driven request expansion.
//!
//! Dispatches multiple requests based on rows from a provided CSV file dataset.
//! Each row in the CSV is used as an 'item' for interpolation in the request.
//!
//! # Examples
//!
//! ```yaml
//! plan:
//!   - name: Fetch users
//!     request:
//!       url: /api/users/{{ item.id }}
//!     with_items_from_csv: users.csv
//! ```

use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_yaml::Value;
use std::path::Path;

use super::pick;
use crate::actions::Request;
use crate::benchmark::Benchmark;
use crate::interpolator::INTERPOLATION_REGEX;
use crate::reader;

/// Checks if the provided YAML item represents a CSV-expanded request action.
///
/// # Arguments
///
/// - `item` (`&Value`) - The YAML item to check
///
/// # Returns
///
/// - `bool` - True if the item is a CSV-expanded request action
///
/// # Examples
///
/// ```rust,ignore
/// use serde_yaml::Value;
/// use floodr::expandable::multi_csv_request;
///
/// let item = serde_yaml::from_str("
/// request:
///   url: /api/users
/// with_items_from_csv: users.csv
/// ").unwrap();
/// assert!(multi_csv_request::is_that_you(&item));
/// ```
pub fn is_that_you(item: &Value) -> bool {
  item.get("request").and_then(|v| v.as_mapping()).is_some() && (item.get("with_items_from_csv").and_then(|v| v.as_str()).is_some() || item.get("with_items_from_csv").and_then(|v| v.as_mapping()).is_some())
}

/// Expands a CSV-expanded request into multiple `Request` actions.
///
/// # Arguments
///
/// - `parent_path` (`&str`) - The path of the parent file, used to resolve the CSV path
/// - `item` (`&Value`) - The YAML item representing the CSV-expanded request
/// - `benchmark` (`&mut Benchmark`) - The benchmark to add the expanded actions to
///
/// # Panics
///
/// - Panics if the CSV file path contains interpolation markers `{{ ... }}`
/// - Panics if `with_items_from_csv` is not correctly specified
///
/// # Examples
///
/// ```rust,ignore
/// use floodr::benchmark::Benchmark;
/// use floodr::expandable::multi_csv_request;
/// use serde_yaml::Value;
///
/// let mut benchmark = Benchmark::new();
/// let item = serde_yaml::from_str("
/// name: Fetch users
/// request:
///   url: /api/users/{{ item.id }}
/// with_items_from_csv: users.csv
/// ").unwrap();
/// multi_csv_request::expand("benchmark.yml", &item, &mut benchmark);
/// ```
pub fn expand(parent_path: &str, item: &Value, benchmark: &mut Benchmark) {
  let (with_items_path, quote_char) = if let Some(with_items_path) = item.get("with_items_from_csv").and_then(|v| v.as_str()) {
    (with_items_path, b'\"')
  } else if let Some(_with_items_hash) = item.get("with_items_from_csv").and_then(|v| v.as_mapping()) {
    let csv_val = item.get("with_items_from_csv").unwrap();
    let with_items_path = csv_val.get("file_name").and_then(|v| v.as_str()).expect("Expected a file_name");
    let quote_char = csv_val.get("quote_char").and_then(|v| v.as_str()).unwrap_or("\"").bytes().next().unwrap();

    (with_items_path, quote_char)
  } else {
    unreachable!();
  };

  if INTERPOLATION_REGEX.is_match(with_items_path) {
    panic!("Interpolations not supported in 'with_items_from_csv' property!");
  }

  let with_items_filepath = Path::new(parent_path).with_file_name(with_items_path);
  let final_path = with_items_filepath.to_str().unwrap();

  let mut with_items_file = reader::read_csv_file_as_yml(final_path, quote_char);

  if let Some(shuffle) = item.get("shuffle").and_then(|v| v.as_bool()) {
    if shuffle {
      let mut rng = thread_rng();
      with_items_file.shuffle(&mut rng);
    }
  }

  let pick = pick(item, &with_items_file);
  for (index, with_item) in with_items_file.iter().take(pick).enumerate() {
    let index = index as u32;

    benchmark.push(Box::new(Request::new(item, Some(with_item.clone()), Some(index))));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn expand_multi() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item.id }}\nwith_items_from_csv: ./fixtures/users.csv";
    let docs = crate::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand("example/benchmark.yml", doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 2);
  }

  #[test]
  fn expand_multi_should_limit_requests_using_the_pick_option() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\npick: 2\nwith_items_from_csv: ./fixtures/users.csv";
    let docs = crate::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand("example/benchmark.yml", doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 2);
  }

  #[test]
  fn expand_multi_should_work_with_pick_and_shuffle() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\npick: 1\nshuffle: true\nwith_items_from_csv: ./fixtures/users.csv";
    let docs = crate::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand("example/benchmark.yml", doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 1);
  }

  #[test]
  #[should_panic]
  fn runtime_expand() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item.id }}\nwith_items_from_csv: ./fixtures/{{ memory }}.csv";
    let docs = crate::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand("example/benchmark.yml", doc, &mut benchmark);
  }
}
