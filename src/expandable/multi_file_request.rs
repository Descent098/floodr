//! File driven request expansion.
//!
//! Expands multiple requests iterating over lines of a text file.
//! Each line is treated as an item for interpolation.
//!
//! # Examples
//!
//! ```yaml
//! plan:
//!   - name: Fetch from list
//!     request:
//!       url: /api/{{ item }}
//!     with_items_from_file: list.txt
//! ```

use super::pick;
use crate::actions::Request;
use crate::benchmark::Benchmark;
use crate::interpolator::INTERPOLATION_REGEX;
use crate::reader;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_yaml::Value;
use std::path::Path;

/// Checks if the provided YAML item represents a file-expanded request action.
///
/// # Arguments
///
/// - `item` (`&Value`) - The YAML item to check
///
/// # Returns
///
/// - `bool` - True if the item is a file-expanded request action
///
/// # Examples
///
/// ```rust,ignore
/// use serde_yaml::Value;
/// use floodr::expandable::multi_file_request;
///
/// let item = serde_yaml::from_str("
/// request:
///   url: /api/{{ item }}
/// with_items_from_file: list.txt
/// ").unwrap();
/// assert!(multi_file_request::is_that_you(&item));
/// ```
pub fn is_that_you(item: &Value) -> bool {
  item.get("request").and_then(|v| v.as_mapping()).is_some() && (item.get("with_items_from_file").and_then(|v| v.as_str()).is_some() || item.get("with_items_from_file").and_then(|v| v.as_mapping()).is_some())
}

/// Expands a file-expanded request into multiple `Request` actions.
///
/// # Arguments
///
/// - `parent_path` (`&str`) - The path of the parent file, used to resolve the text file path
/// - `item` (`&Value`) - The YAML item representing the file-expanded request
/// - `benchmark` (`&mut Benchmark`) - The benchmark to add the expanded actions to
///
/// # Panics
///
/// - Panics if the text file path contains interpolation markers `{{ ... }}`
///
/// # Examples
///
/// ```rust,ignore
/// use floodr::benchmark::Benchmark;
/// use floodr::expandable::multi_file_request;
/// use serde_yaml::Value;
///
/// let mut benchmark = Benchmark::new();
/// let item = serde_yaml::from_str("
/// name: Fetch from list
/// request:
///   url: /api/{{ item }}
/// with_items_from_file: list.txt
/// ").unwrap();
/// multi_file_request::expand("benchmark.yml", &item, &mut benchmark);
/// ```
pub fn expand(parent_path: &str, item: &Value, benchmark: &mut Benchmark) {
  let with_items_path = if let Some(with_items_path) = item.get("with_items_from_file").and_then(|v| v.as_str()) {
    with_items_path
  } else {
    unreachable!();
  };

  if INTERPOLATION_REGEX.is_match(with_items_path) {
    panic!("Interpolation not supported in 'with_items_from_file' property!");
  }

  let with_items_filepath = Path::new(parent_path).with_file_name(with_items_path);
  let final_path = with_items_filepath.to_str().unwrap();

  let mut with_items_file = reader::read_file_as_yml_array(final_path);

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
mod test {
  use super::*;

  #[test]
  fn expand_multi() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item.id }}\nwith_items_from_file: ./fixtures/texts.txt";
    let docs = crate::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand("example/benchmark.yml", doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 3);
  }

  #[test]
  fn expand_multi_should_limit_requests_using_the_pick_option() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\npick: 2\nwith_items_from_file: ./fixtures/texts.txt";
    let docs = crate::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand("example/benchmark.yml", doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 2);
  }

  #[test]
  fn expand_multi_should_work_with_pick_and_shuffle() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\npick: 1\nshuffle: true\nwith_items_from_file: ./fixtures/texts.txt";
    let docs = crate::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand("example/benchmark.yml", doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 1);
  }
}
