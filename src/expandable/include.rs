//! Include action expansion handling.
//!
//! Handles expanding `include: file.yml` directives from benchmark plans.
//! This allows for modularizing benchmark files and reusing parts of a plan.
//!
//! # Examples
//!
//! ```yaml
//! plan:
//!   - name: Include external plan
//!     include: auth_steps.yml
//! ```

use serde_yaml::Value;
use std::path::Path;

use crate::parsing::interpolator::INTERPOLATION_REGEX;

use crate::actions;
use crate::engine::benchmark::Benchmark;
use crate::expandable::{include, multi_csv_request, multi_file_request, multi_iter_request, multi_request};
use crate::parsing::tags::Tags;

use crate::parsing::reader;

/// Checks if the provided YAML item represents an `include` action.
///
/// # Arguments
///
/// - `item` (`&Value`) - The YAML item to check
///
/// # Returns
///
/// - `bool` - True if the item is an include directive
///
/// # Examples
///
/// ```rust,ignore
/// use serde_yaml::Value;
/// use floodr::expandable::include;
///
/// let item = serde_yaml::from_str("include: auth.yml").unwrap();
/// assert!(include::is_that_you(&item));
/// ```
pub fn is_that_you(item: &Value) -> bool {
  item.get("include").and_then(|v| v.as_str()).is_some()
}

/// Expands an `include` action by reading the specified file and adding its contents to the benchmark.
///
/// # Arguments
///
/// - `parent_path` (`&str`) - The path of the parent file, used to resolve relative paths
/// - `item` (`&Value`) - The YAML item representing the include action
/// - `benchmark` (`&mut Benchmark`) - The benchmark to add the expanded actions to
/// - `tags` (`&Tags`) - The tags to filter the included items
///
/// # Panics
///
/// - Panics if the `include` path contains interpolation markers `{{ ... }}`
///
/// # Examples
///
/// ```rust,ignore
/// use floodr::benchmark::Benchmark;
/// use floodr::tags::Tags;
/// use floodr::expandable::include;
/// use serde_yaml::Value;
///
/// let mut benchmark = Benchmark::new();
/// let item = serde_yaml::from_str("include: tests.yml").unwrap();
/// include::expand("benchmark.yml", &item, &mut benchmark, &Tags::new(None, None));
/// ```
pub fn expand(parent_path: &str, item: &Value, benchmark: &mut Benchmark, tags: &Tags) {
  let include_path = item.get("include").and_then(|v| v.as_str()).unwrap();

  if INTERPOLATION_REGEX.is_match(include_path) {
    panic!("Interpolations not supported in 'include' property!");
  }

  let include_filepath = Path::new(parent_path).with_file_name(include_path);
  let final_path = include_filepath.to_str().unwrap();

  expand_from_filepath(final_path, benchmark, None, tags);
}

/// Reads a benchmark file from a path and expands its contents into the provided benchmark.
///
/// This handles nested includes and various action types (request, delay, exec, etc.).
///
/// # Arguments
///
/// - `parent_path` (`&str`) - The path of the file to read
/// - `benchmark` (`&mut Benchmark`) - The benchmark to add the expanded actions to
/// - `accessor` (`Option<&str>`) - Optional YAML accessor to read a sub-property
/// - `tags` (`&Tags`) - The tags to filter the items
///
/// # Panics
///
/// - Panics if it encounters an unknown action type
///
/// # Examples
///
/// ```rust,ignore
/// use floodr::benchmark::Benchmark;
/// use floodr::tags::Tags;
/// use floodr::expandable::include;
///
/// let mut benchmark = Benchmark::new();
/// include::expand_from_filepath("auth.yml", &mut benchmark, None, &Tags::new(None, None));
/// ```
pub fn expand_from_filepath(parent_path: &str, benchmark: &mut Benchmark, accessor: Option<&str>, tags: &Tags) {
  let docs = reader::read_file_as_yml(parent_path);
  let items = reader::read_yaml_doc_accessor(&docs[0], accessor);

  for item in items {
    if include::is_that_you(item) {
      include::expand(parent_path, item, benchmark, tags);

      continue;
    }

    if tags.should_skip_item(item) {
      continue;
    }

    if multi_request::is_that_you(item) {
      multi_request::expand(item, benchmark);
    } else if multi_iter_request::is_that_you(item) {
      multi_iter_request::expand(item, benchmark);
    } else if multi_csv_request::is_that_you(item) {
      multi_csv_request::expand(parent_path, item, benchmark);
    } else if multi_file_request::is_that_you(item) {
      multi_file_request::expand(parent_path, item, benchmark);
    } else if actions::Delay::is_that_you(item) {
      benchmark.push(Box::new(actions::Delay::new(item, None)));
    } else if actions::Exec::is_that_you(item) {
      benchmark.push(Box::new(actions::Exec::new(item, None)));
    } else if actions::Assign::is_that_you(item) {
      benchmark.push(Box::new(actions::Assign::new(item, None)));
    } else if actions::Assert::is_that_you(item) {
      benchmark.push(Box::new(actions::Assert::new(item, None)));
    } else if actions::Request::is_that_you(item) {
      benchmark.push(Box::new(actions::Request::new(item, None, None)));
    } else {
      let out_str = serde_yaml::to_string(item).unwrap();
      panic!("Unknown node:\n\n{out_str}\n\n");
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::engine::benchmark::Benchmark;
  use crate::expandable::include::{expand, is_that_you};
  use crate::parsing::tags::Tags;

  #[test]
  fn expand_include() {
    let text = "---\nname: Include comment\ninclude: comments.yml";
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand("example/benchmark.yml", doc, &mut benchmark, &Tags::new(None, None));

    assert!(is_that_you(doc));
    assert_eq!(benchmark.len(), 2);
  }

  #[test]
  #[should_panic]
  fn invalid_expand() {
    let text = "---\nname: Include comment\ninclude: {{ memory }}.yml";
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    let doc = &docs[0];
    let mut benchmark: Benchmark = Benchmark::new();

    expand("example/benchmark.yml", doc, &mut benchmark, &Tags::new(None, None));
  }
}
