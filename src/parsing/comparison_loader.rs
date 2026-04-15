//! Loader module for reconstructing benchmark plans from report files to use for comparisons
//! 
//! # Examples
//! 
//! With the report file:
//! 
//! `rep.yml`
//! 
//! ```yaml
//! # An example report
//! base: http://localhost:4896
//! baseline:
//! - duration: 92.64867
//!   name: Fetch route
//!   status: 200
//! plan:
//! - assign: gothamServer
//!   name: Fetch route
//!   request:
//!     url: /
//! ```
//! 
//! You would get:
//! 
//! ```
//! let (base, plan_items, report_baseline) = floodr::parsing::comparison_loader::load_report_data("example/rep.yml");
//! 
//! println!("{}", base); // http://localhost:4896
//! 
//! println!("{:#?}", plan_items);
//! // [
//! //     Mapping {
//! //         "assign": String("gothamServer"),
//! //         "name": String("Fetch route"),
//! //         "request": Mapping {
//! //             "url": String("/"),
//! //         },
//! //     },
//! // ]
//! 
//! println!("{:#?}", report_baseline);
//! // [
//! // - name: Fetch route
//! //   duration: 92.64867
//! // ]
//! ```



use serde_yaml::Value;
use crate::engine::benchmark::{ActionItem, Benchmark};
use crate::actions;
use crate::parsing::reader;

/// Loads report data from a YAML file.
///
/// # Arguments
///
/// - `filepath` (`&str`) - Path to the report file
///
/// # Returns
///
/// - `(String, Vec<Value>, Vec<actions::Report>)` - The base URL, the plan items, and the baseline results
/// 
/// # Examples
/// 
/// With the report file:
/// 
/// `rep.yml`
/// 
/// ```yaml
/// # An example report
/// base: http://localhost:4896
/// baseline:
/// - duration: 92.64867
///   name: Fetch route
///   status: 200
/// plan:
/// - assign: gothamServer
///   name: Fetch route
///   request:
///     url: /
/// ```
/// 
/// You would get:
/// 
/// ```
/// let (base, plan_items, report_baseline) = floodr::parsing::comparison_loader::load_report_data("example/rep.yml");
/// 
/// println!("{}", base); // http://localhost:4896
/// 
/// println!("{:#?}", plan_items);
/// // [
/// //     Mapping {
/// //         "assign": String("gothamServer"),
/// //         "name": String("Fetch route"),
/// //         "request": Mapping {
/// //             "url": String("/"),
/// //         },
/// //     },
/// // ]
/// 
/// println!("{:#?}", report_baseline);
/// // [
/// // - name: Fetch route
/// //   duration: 92.64867
/// // ]
/// ```
/// 
pub fn load_report_data(filepath: &str) -> (String, Vec<Value>, Vec<actions::Report>) {
  let docs = reader::read_file_as_yml(filepath);
  let doc = &docs[0];

  let base = doc.get("base").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let plan = doc.get("plan").and_then(|v| v.as_sequence()).cloned().unwrap_or_default();
  
  let baseline_val = doc.get("baseline").cloned().unwrap_or(Value::Sequence(vec![]));
  let baseline: Vec<actions::Report> = serde_yaml::from_value(baseline_val).expect("Failed to parse baseline from report");

  (base, plan, baseline)
}

/// Reconstructs a benchmark plan from a list of YAML items.
///
/// This function creates runnable actions directly from the provided items
/// without performing any further expansion (like includes or multi-request).
///
/// # Arguments
///
/// - `items` (`Vec<Value>`) - The YAML items representing the flattened plan
///
/// # Returns
///
/// - `Benchmark` - The reconstructed benchmark plan
/// 
/// # Examples
/// 
/// With the report file:
/// 
/// `rep.yml`
/// 
/// ```yaml
/// # An example report
/// base: http://localhost:4896
/// baseline:
/// - duration: 92.64867
///   name: Fetch route
///   status: 200
/// plan:
/// - assign: gothamServer
///   name: Fetch route
///   request:
///     url: /
/// ```
/// 
/// You would get:
/// ```
/// let (base, plan_items, report_baseline) = floodr::parsing::comparison_loader::load_report_data("example/rep.yml");
/// 
/// let benchmark_plan = floodr::parsing::comparison_loader::load_from_items(plan_items);
///```
/// 
pub fn load_from_items(items: Vec<Value>) -> Benchmark {
  let mut benchmark = Benchmark::new();

  for item in items {
    if actions::Delay::is_that_you(&item) {
      benchmark.push(ActionItem::new(Box::new(actions::Delay::new(&item, None)), item.clone()));
    } else if actions::Exec::is_that_you(&item) {
      benchmark.push(ActionItem::new(Box::new(actions::Exec::new(&item, None)), item.clone()));
    } else if actions::Assign::is_that_you(&item) {
      benchmark.push(ActionItem::new(Box::new(actions::Assign::new(&item, None)), item.clone()));
    } else if actions::Assert::is_that_you(&item) {
      benchmark.push(ActionItem::new(Box::new(actions::Assert::new(&item, None)), item.clone()));
    } else if actions::Request::is_that_you(&item) {
      benchmark.push(ActionItem::new(Box::new(actions::Request::new(&item, None, None)), item.clone()));
    } else {
      let out_str = serde_yaml::to_string(&item).unwrap();
      panic!("Unknown node in report plan:\n\n{out_str}\n\n");
    }
  }

  benchmark
}
