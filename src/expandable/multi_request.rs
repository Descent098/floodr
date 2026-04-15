//! Simple list driven request expansion.
//!
//! Expands requests iterating over a literal list provided in the YAML.
//! Each item in the list is used for interpolation in the request.
//!
//! # Examples
//!
//! ```yaml
//! plan:
//!   - name: Fetch items
//!     request:
//!       url: /api/{{ item }}
//!     with_items:
//!       - alpha
//!       - beta
//!       - gamma
//! ```

use rand::seq::SliceRandom;
use serde_yaml::Value;

use super::pick;
use crate::actions::Request;
use crate::engine::benchmark::{ActionItem, Benchmark};
use crate::parsing::interpolator::INTERPOLATION_REGEX;

/// Checks if the provided YAML item represents a literal list-expanded request action.
///
/// # Arguments
///
/// - `item` (`&Value`) - The YAML item to check
///
/// # Returns
///
/// - `bool` - True if the item is a list-expanded request action
///
/// # Examples
///
/// ```rust,ignore
/// use serde_yaml::Value;
/// use floodr::expandable::multi_request;
///
/// let item = serde_yaml::from_str("
/// request:
///   url: /api/{{ item }}
/// with_items:
///   - item1
///   - item2
/// ").unwrap();
/// assert!(multi_request::is_that_you(&item));
/// ```
pub fn is_that_you(item: &Value) -> bool {
  item.get("request").and_then(|v| v.as_mapping()).is_some() && item.get("with_items").and_then(|v| v.as_sequence()).is_some()
}

/// Expands a literal list-expanded request into multiple `Request` actions.
///
/// # Arguments
///
/// - `item` (`&Value`) - The YAML item representing the list-expanded request
/// - `benchmark` (`&mut Benchmark`) - The benchmark to add the expanded actions to
///
/// # Panics
///
/// - Panics if any of the `with_items` children contain interpolation markers `{{ ... }}`
///
/// # Examples
///
/// ```rust,ignore
/// use floodr::benchmark::Benchmark;
/// use floodr::expandable::multi_request;
/// use serde_yaml::Value;
///
/// let mut benchmark = Benchmark::new();
/// let item = serde_yaml::from_str("
/// name: Fetch items
/// request:
///   url: /api/{{ item }}
/// with_items:
///   - a
///   - b
/// ").unwrap();
/// multi_request::expand(&item, &mut benchmark);
/// ```
pub fn expand(item: &Value, benchmark: &mut Benchmark) {
  if let Some(with_items) = item.get("with_items").and_then(|v| v.as_sequence()) {
    let mut with_items_list = with_items.clone();

    if let Some(shuffle) = item.get("shuffle").and_then(|v| v.as_bool()) && shuffle {
        let mut rng = rand::rng();
        with_items_list.shuffle(&mut rng);
    }

    let pick = pick(item, &with_items_list);
    for (index, with_item) in with_items_list.iter().take(pick).enumerate() {
      let index = index as u32;

      let value: &str = with_item.as_str().unwrap_or("");

      if INTERPOLATION_REGEX.is_match(value) {
        panic!("Interpolations not supported in 'with_items' children!");
      }

      let mut source = item.clone();
      if let Some(map) = source.as_mapping_mut() {
        map.insert(serde_yaml::Value::String("with_item".to_string()), with_item.clone());
        map.insert(serde_yaml::Value::String("index".to_string()), serde_yaml::Value::Number(index.into()));
        map.remove(&serde_yaml::Value::String("with_items".to_string()));
      }

      benchmark.push(ActionItem::new(Box::new(Request::new(item, Some(with_item.clone()), Some(index))), source));
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn expand_multi() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\nwith_items:\n  - 1\n  - 2\n  - 3";
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand(doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 3);
  }

  #[test]
  fn expand_multi_should_limit_requests_using_the_pick_option() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\npick: 2\nwith_items:\n  - 1\n  - 2\n  - 3";
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand(doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 2);
  }

  #[test]
  fn expand_multi_should_work_with_pick_and_shuffle() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\npick: 1\nshuffle: true\nwith_items:\n  - 1\n  - 2\n  - 3";
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand(doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 1);
  }

  #[test]
  #[should_panic]
  fn runtime_expand() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\nwith_items:\n  - 1\n  - 2\n  - foo{{ memory }}";
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand(doc, &mut benchmark);
  }
}
