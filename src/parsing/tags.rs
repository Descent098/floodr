//! Tag filtering logic.
//!
//! Exposes structures and functionality to filter out or force include
//! benchmark items based on their associated tags ( Ansible-style tags ).
//! # Example
//! 
//! ```
//! 
//! let tags = floodr::parsing::tags::Tags::new(Some("requests"), None);
//! 
//! 
//! let fetch_src = r#"
//! name: Fetch
//! request:
//!   url: /
//! tags: ["requests"]
//! assign: val
//! "#;
//! let fetch_data = serde_yaml::from_str(fetch_src).expect("Failed to parse");
//! 
//! println!("{}",tags.should_skip_item(&fetch_data)); // false
//! 
//! let assert_src = r#"
//! name: assert response
//! assert:
//!   key: val.status
//!   value: 200
//! tags: ["asserts"]
//! "#;
//! let assert_data = serde_yaml::from_str(assert_src).expect("Failed to parse");
//! 
//! println!("{}",tags.should_skip_item(&assert_data)); //true
//! ```
use crate::parsing::reader;
use colored::*;
use serde_yaml::Value;
use std::collections::HashSet;

/// Represents a set of include and exclude filters (tags).
///
/// Tags allow for choosing exactly which parts of a benchmark to execute.
///
/// # Fields
///
/// - `tags` (`Option<HashSet<&'a str>>`) - A set of tags to include.
/// - `skip_tags` (`Option<HashSet<&'a str>>`) - A set of tags to exclude.
#[derive(Debug)]
pub struct Tags<'a> {
  pub tags: Option<HashSet<&'a str>>,
  pub skip_tags: Option<HashSet<&'a str>>,
}

impl<'a> Tags<'a> {
  /// Initializes a new `Tags` instance from comma-separated string options.
  ///
  /// # Arguments
  ///
  /// - `tags_option` (`Option<&'a str>`) - A comma-separated string of tags to include.
  /// - `skip_tags_option` (`Option<&'a str>`) - A comma-separated string of tags to skip.
  ///
  /// # Returns
  ///
  /// - `Self` - A new `Tags` instance.
  ///
  /// # Panics
  ///
  /// - Panics if both `tags` and `skip-tags` share any common tag values.
  ///
  /// # Examples
  ///
  /// ```rust
  /// let tags = floodr::parsing::tags::Tags::new(Some("tag1,tag2"), Some("tag3"));
  /// ```
  pub fn new(tags_option: Option<&'a str>, skip_tags_option: Option<&'a str>) -> Self {
    let tags: Option<HashSet<&str>> = tags_option.map(|m| m.split(',').map(|s| s.trim()).collect());
    let skip_tags: Option<HashSet<&str>> = skip_tags_option.map(|m| m.split(',').map(|s| s.trim()).collect());

    if let (Some(t), Some(s)) = (&tags, &skip_tags) && !t.is_disjoint(s) {
        panic!("`tags` and `skip-tags` must not contain the same values!");
    }


    Tags {
      tags,
      skip_tags,
    }
  }

  /// Determines if a benchmark item should be skipped based on its tags.
  ///
  /// Following Ansible-style logic:
  /// - Items with `always` are run unless explicitly skipped.
  /// - Items with `never` are skipped unless explicitly included.
  /// - If `tags` are provided, only items with matching tags (or `always`) are run.
  /// - If `skip_tags` are provided, items matching those tags are skipped.
  ///
  /// # Arguments
  ///
  /// - `item` (`&Value`) - The YAML representation of the benchmark action.
  ///
  /// # Returns
  ///
  /// - `bool` - True if the item should be skipped, false otherwise.
  /// 
  /// # Example
  /// 
  /// ```
  /// let tags = floodr::parsing::tags::Tags::new(Some("requests"), None);
  /// 
  /// 
  /// let fetch_src = r#"
  /// name: Fetch
  /// request:
  ///   url: /
  /// tags: ["requests"]
  /// assign: val
  /// "#;
  /// let fetch_data = serde_yaml::from_str(fetch_src).expect("Failed to parse");
  /// 
  /// println!("{}",tags.should_skip_item(&fetch_data)); // false
  /// 
  /// let assert_src = r#"
  /// name: assert response
  /// assert:
  ///   key: val.status
  ///   value: 200
  /// tags: ["asserts"]
  /// "#;
  /// let assert_data = serde_yaml::from_str(assert_src).expect("Failed to parse");
  /// 
  /// println!("{}",tags.should_skip_item(&assert_data)); //true
  /// ```
  pub fn should_skip_item(&self, item: &Value) -> bool {
    match item.get("tags").and_then(|v| v.as_sequence()) {
      Some(item_tags_raw) => {
        let item_tags: HashSet<&str> = item_tags_raw.iter().filter_map(|t| t.as_str()).collect();
        if let Some(s) = &self.skip_tags && !s.is_disjoint(&item_tags){
            return true;
        }
        if let Some(t) = &self.tags {
          if item_tags.contains("never") && !t.contains("never") {
            return true;
          }
          if !t.is_disjoint(&item_tags) {
            return false;
          }
        }
        if item_tags.contains("always") {
          return false;
        }
        if item_tags.contains("never") {
          return true;
        }
        self.tags.is_some()
      }
      None => self.tags.is_some(),
    }
  }
}

/// Lists and prints all benchmark tasks (actions) that would be executed given the filters.
///
/// # Arguments
///
/// - `benchmark_file` (`&str`) - Path to the benchmark YAML file.
/// - `tags` (`&Tags`) - The current tag filters to apply when listing tasks.
///
/// # Panics
///
/// - Panics if no items remain after filtering.
/// 
/// # Example
/// 
/// With the file
/// 
/// `tags.yml`
/// ```yaml
/// base: http://localhost:9000
/// 
/// plan:
///   - name: Fetch
///     request:
///       url: /
///     tags:
///       - requests
///     assign: val
///   - name: assert response
///     assert:
///       key: val.status
///       value: 200
///     tags: ["asserts"]
/// ```
/// 
/// You can then run:
/// 
/// ```
/// let tags = floodr::parsing::tags::Tags::new(Some("requests,asserts"), None);
/// // Prints: "Tags            ["asserts", "requests"]"
/// 
/// let data = floodr::parsing::tags::list_benchmark_file_tasks("example/tags.yml", &tags);
/// 
/// // Prints:
/// //
/// // name: Fetch
/// // request:
/// //   url: /
/// // tags:
/// // - requests
/// // assign: val
/// // 
/// // name: assert response
/// // assert:
/// //   key: val.status
/// //   value: 200
/// // tags:
/// // - asserts
/// ```
pub fn list_benchmark_file_tasks(benchmark_file: &str, tags: &Tags) {
  let docs = reader::read_file_as_yml(benchmark_file);
  let items = reader::read_yaml_doc_accessor(&docs[0], Some("plan"));

  println!();

  if let Some(tags) = &tags.tags {
    let mut tags: Vec<_> = tags.iter().collect();
    tags.sort();
    println!("{:width$} {:width2$?}", "Tags".green(), &tags, width = 15, width2 = 25);
  }
  if let Some(tags) = &tags.skip_tags {
    let mut tags: Vec<_> = tags.iter().collect();
    tags.sort();
    println!("{:width$} {:width2$?}", "Skip-Tags".green(), &tags, width = 15, width2 = 25);
  }

  let items: Vec<_> = items.iter().filter(|item| !tags.should_skip_item(item)).collect();

  if items.is_empty() {
    println!("{}", "No items".red());
    std::process::exit(1)
  }

  for item in items {
    let out_str = serde_yaml::to_string(item).unwrap();
    println!("{out_str}");
  }
}

/// Lists and prints all unique tags found within a benchmark file's plan.
///
/// # Arguments
///
/// - `benchmark_file` (`&str`) - Path to the benchmark YAML file.
///
/// # Panics
///
/// - Panics if the benchmark file contains no plan items.
/// 
/// # Example
/// 
/// With the file
/// 
/// `tags.yml`
/// ```yaml
/// base: http://localhost:9000
/// 
/// plan:
///   - name: Fetch
///     request:
///       url: /
///     tags:
///       - requests
///     assign: val
///   - name: assert response
///     assert:
///       key: val.status
///       value: 200
///     tags: ["asserts"]
/// ```
/// 
/// You can then run:
/// 
/// ```
/// let data = floodr::parsing::tags::list_benchmark_file_tags("example/tags.yml");
/// // prints: "Tags            ["asserts", "requests"]"
/// ```
pub fn list_benchmark_file_tags(benchmark_file: &str) {
  let docs = reader::read_file_as_yml(benchmark_file);
  let items = reader::read_yaml_doc_accessor(&docs[0], Some("plan"));

  println!();

  if items.is_empty() {
    println!("{}", "No items".red());
    std::process::exit(1)
  }
  let mut tags: HashSet<&str> = HashSet::new();
  for item in items {
    if let Some(item_tags_raw) = item.get("tags").and_then(|v| v.as_sequence()) {
      tags.extend(item_tags_raw.iter().filter_map(|t| t.as_str()));
    }
  }

  let mut tags: Vec<_> = tags.into_iter().collect();
  tags.sort_unstable();
  println!("{:width$} {:?}", "Tags".green(), &tags, width = 15);
}

#[cfg(test)]
mod tests {
  use super::*;

  fn str_to_yaml(text: &str) -> Value {
    let docs = crate::parsing::reader::read_file_as_yml_from_str(text);
    docs[0].clone()
  }

  fn prepare_default_item() -> Value {
    str_to_yaml("---\nname: foo\nrequest:\n  url: /\ntags:\n  - tag1\n  - tag2")
  }

  #[test]
  #[should_panic]
  fn same_tags_and_skip_tags() {
    let _ = Tags::new(Some("tag1"), Some("tag1"));
  }

  #[test]
  fn empty_tags_both() {
    let item = str_to_yaml("---\nname: foo\nrequest:\n  url: /");
    let tags = Tags::new(None, None);
    assert!(!tags.should_skip_item(&item));
  }

  #[test]
  fn empty_tags() {
    let tags = Tags::new(None, None);
    assert!(!tags.should_skip_item(&prepare_default_item()));
  }

  #[test]
  fn tags_contains() {
    let tags = Tags::new(Some("tag1"), None);
    assert!(!tags.should_skip_item(&prepare_default_item()));
  }

  #[test]
  fn tags_contains_second() {
    let tags = Tags::new(Some("tag2"), None);
    assert!(!tags.should_skip_item(&prepare_default_item()));
  }

  #[test]
  fn tags_contains_both() {
    let tags = Tags::new(Some("tag1,tag2"), None);
    assert!(!tags.should_skip_item(&prepare_default_item()));
  }

  #[test]
  fn tags_not_contains() {
    let tags = Tags::new(Some("tag99"), None);
    assert!(tags.should_skip_item(&prepare_default_item()));
  }

  #[test]
  fn skip_tags_not_contains() {
    let tags = Tags::new(None, Some("tag99"));
    assert!(!tags.should_skip_item(&prepare_default_item()));
  }

  #[test]
  fn skip_tags_contains() {
    let tags = Tags::new(None, Some("tag1"));
    assert!(tags.should_skip_item(&prepare_default_item()));
  }

  #[test]
  fn skip_tags_contains_second() {
    let tags = Tags::new(None, Some("tag2"));
    assert!(tags.should_skip_item(&prepare_default_item()));
  }

  #[test]
  fn tags_contains_but_also_skip_tags_contains() {
    let tags = Tags::new(Some("tag1"), Some("tag2"));
    assert!(tags.should_skip_item(&prepare_default_item()));
  }

  #[test]
  fn never_skipped_by_default() {
    let item = str_to_yaml("---\nname: foo\nrequest:\n  url: /\ntags:\n  - never\n  - tag2");
    let tags = Tags::new(None, None);
    assert!(tags.should_skip_item(&item));
  }

  #[test]
  fn never_tag_skipped_even_when_other_tag_included() {
    let item = str_to_yaml("---\nname: foo\nrequest:\n  url: /\ntags:\n  - never\n  - tag2");
    let tags = Tags::new(Some("tag2"), None);
    assert!(tags.should_skip_item(&item));
  }

  #[test]
  fn include_never_tag() {
    let item = str_to_yaml("---\nname: foo\nrequest:\n  url: /\ntags:\n  - never\n  - tag2");
    let tags = Tags::new(Some("never"), None);
    assert!(!tags.should_skip_item(&item));
  }

  #[test]
  fn always_tag_included_by_default() {
    let item = str_to_yaml("---\nname: foo\nrequest:\n  url: /\ntags:\n  - always\n  - tag2");
    let tags = Tags::new(Some("tag99"), None);
    assert!(!tags.should_skip_item(&item));
  }

  #[test]
  fn skip_always_tag() {
    let item = str_to_yaml("---\nname: foo\nrequest:\n  url: /\ntags:\n  - always\n  - tag2");
    let tags = Tags::new(None, Some("always"));
    assert!(tags.should_skip_item(&item));
  }
}
