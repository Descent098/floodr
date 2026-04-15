//! Range driven request expansion.
//!
//! Allows dispatching multiple requests by iterating over an integer range.
//!
//! # Examples
//!
//! ```yaml
//! plan:
//!   - name: Sequential requests
//!     request:
//!       url: /api/items/{{ item }}
//!     with_items_range:
//!       start: 1
//!       stop: 10
//!       step: 1
//! ```

use std::convert::TryInto;

use rand::seq::SliceRandom;
use serde_yaml::{Number, Value};

use crate::parsing::interpolator::INTERPOLATION_REGEX;

use crate::actions::Request;
use crate::engine::benchmark::{ActionItem, Benchmark};

/// Checks if the provided YAML item represents a range-expanded request action.
///
/// # Arguments
///
/// - `item` (`&Value`) - The YAML item to check
///
/// # Returns
///
/// - `bool` - True if the item is a range-expanded request action
///
/// # Examples
///
/// ```rust,ignore
/// use serde_yaml::Value;
/// use floodr::expandable::multi_iter_request;
///
/// let item = serde_yaml::from_str("
/// request:
///   url: /api/{{ item }}
/// with_items_range:
///   start: 1
///   stop: 5
/// ").unwrap();
/// assert!(multi_iter_request::is_that_you(&item));
/// ```
pub fn is_that_you(item: &Value) -> bool {
  item.get("request").and_then(|v| v.as_mapping()).is_some() && item.get("with_items_range").and_then(|v| v.as_mapping()).is_some()
}

/// Expands a range-expanded request into multiple `Request` actions.
///
/// # Arguments
///
/// - `item` (`&Value`) - The YAML item representing the range-expanded request
/// - `benchmark` (`&mut Benchmark`) - The benchmark to add the expanded actions to
///
/// # Panics
///
/// - Panics if `start`, `step`, or `stop` contain interpolation markers `{{ ... }}`
/// - Panics if `start` or `stop` properties are missing
/// - Panics if `start`, `step`, or `stop` are not numbers
///
/// # Examples
///
/// ```rust,ignore
/// use floodr::benchmark::Benchmark;
/// use floodr::expandable::multi_iter_request;
/// use serde_yaml::Value;
///
/// let mut benchmark = Benchmark::new();
/// let item = serde_yaml::from_str("
/// name: Sequential requests
/// request:
///   url: /api/items/{{ item }}
/// with_items_range:
///   start: 1
///   stop: 10
/// ").unwrap();
/// multi_iter_request::expand(&item, &mut benchmark);
/// ```
pub fn expand(item: &Value, benchmark: &mut Benchmark) {
  if let Some(with_iter_items) = item.get("with_items_range").and_then(|v| v.as_mapping()) {
    let lstart = Value::String("start".into());
    let lstep = Value::String("step".into());
    let lstop = Value::String("stop".into());

    let vstart = with_iter_items.get(&lstart).expect("Start property is mandatory");
    let default_step = Value::Number(Number::from(1));
    let vstep = with_iter_items.get(&lstep).unwrap_or(&default_step);
    let vstop = with_iter_items.get(&lstop).expect("Stop property is mandatory");

    let start: &str = vstart.as_str().unwrap_or("");
    let step: &str = vstep.as_str().unwrap_or("");
    let stop: &str = vstop.as_str().unwrap_or("");

    if INTERPOLATION_REGEX.is_match(start) {
      panic!("Interpolations not supported in 'start' property!");
    }

    if INTERPOLATION_REGEX.is_match(step) {
      panic!("Interpolations not supported in 'step' property!");
    }

    if INTERPOLATION_REGEX.is_match(stop) {
      panic!("Interpolations not supported in 'stop' property!");
    }

    let start: i64 = vstart.as_i64().expect("Start needs to be a number");
    let step: i64 = vstep.as_i64().expect("Step needs to be a number");
    let stop: i64 = vstop.as_i64().expect("Stop needs to be a number");

    let stop = stop + 1; // making stop inclusive

    if stop > start && start > 0 {
      let mut with_items: Vec<i64> = (start..stop).step_by(step as usize).collect();

      if let Some(shuffle) = item.get("shuffle").and_then(|v| v.as_bool()) && shuffle {
          let mut rng = rand::rng();
          with_items.shuffle(&mut rng);
        }

      if let Some(pick) = item.get("pick").and_then(|v| v.as_i64()) {
        with_items.truncate(pick.try_into().expect("pick can't be larger than size of range"))
      }

      for (index, value) in with_items.iter().enumerate() {
        let index = index as u32;

        let mut source = item.clone();
        if let Some(map) = source.as_mapping_mut() {
          map.insert(Value::String("with_item".into()), Value::Number(Number::from(*value)));
          map.insert(Value::String("index".into()), Value::Number(Number::from(index)));
          map.remove(&Value::String("with_items_range".into()));
        }

        benchmark.push(ActionItem::new(Box::new(Request::new(item, Some(Value::Number(Number::from(*value))), Some(index))), source));
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn expand_multi_range() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\nwith_items_range:\n  start: 2\n  step: 2\n  stop: 20";
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand(doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 10);
  }

  #[test]
  fn expand_multi_range_should_limit_requests_using_the_pick_option() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\npick: 3\nwith_items_range:\n  start: 2\n  step: 2\n  stop: 20";
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand(doc, &mut benchmark);

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 3);
  }

  #[test]
  #[should_panic]
  fn invalid_expand() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\nwith_items_range:\n  start: 1\n  step: 2\n  stop: foo";
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand(doc, &mut benchmark);
  }

  #[test]
  #[should_panic]
  fn runtime_expand() {
    let text = "---\nname: foobar\nrequest:\n  url: /api/{{ item }}\nwith_items_range:\n  start: 1\n  step: 2\n  stop: \"{{ memory }}\"";
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand(doc, &mut benchmark);
  }
}
