//! Compare executions of a benchmark against a previous one.
//!
//! Validates if the current runs are within the threshold difference
//! of the previously recorded baseline metrics.
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
//! ```rust
//! let current_run = vec![
//!     vec![
//!         floodr::actions::Report {
//!             name: "Fetch account".to_string(),
//!             duration: 115.0,
//!             status: 200,
//!         },
//!     ]
//! ];
//! 
//! let result = floodr::parsing::checker::compare(&current_run, "example/rep.yml", &3.to_string());
//! 
//! // would print: "Fetch account             is 22ms slower than before"
//! 
//! match result {
//!     Ok(_) => println!("No values above threshold"),
//!     Err(count) => println!("{} Values over threshold", count) // "1 Values over threshold" would print
//! };
//! 
//! ```
//! 

use colored::*;

use crate::actions::Report;
use crate::parsing::reader;

/// Extracts the base URL from a report file.
///
/// # Arguments
///
/// - `filepath` (`&str`) - Path to the report YAML file.
///
/// # Returns
///
/// - `String` - The base URL found in the report.
/// 
/// # Example
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
/// ```rust
/// let base = floodr::parsing::checker::get_base("example/rep.yml");
/// println!("{}", base); // http://localhost:4896
/// ```
pub fn get_base(filepath: &str) -> String {
  let docs = reader::read_file_as_yml(filepath);
  let doc = &docs[0];
  doc.get("base").and_then(|v| v.as_str()).unwrap_or("").to_string()
}

/// Compares the results of a benchmark run against a previous execution's baseline.
///
/// It iterates through each request in the reports and checks if the duration
/// exceeds the baseline duration by more than the specified threshold.
///
/// # Arguments
///
/// - `list_reports` (`&[Vec<Report>]`) - The current benchmark results (multi-iteration).
/// - `filepath` (`&str`) - Path to the baseline YAML report file.
/// - `threshold` (`&str`) - The maximum allowed duration increase in milliseconds.
///
/// # Returns
///
/// - `Result<(), i32>` - `Ok(())` if all requests are within threshold, otherwise `Err(count)` where `count` is the number of slow requests.
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
/// ```rust
/// let current_run = vec![
///     vec![
///         floodr::actions::Report {
///             name: "Fetch account".to_string(),
///             duration: 115.0,
///             status: 200,
///         },
///     ]
/// ];
/// 
/// let result = floodr::parsing::checker::compare(&current_run, "example/rep.yml", &3.to_string());
/// 
/// // would print: "Fetch account             is 22ms slower than before"
/// 
/// match result {
///     Ok(_) => println!("No values above threshold"),
///     Err(count) => println!("{} Values over threshold", count) // "1 Values over threshold" would print
/// };
/// 
/// ```
pub fn compare(list_reports: &[Vec<Report>], filepath: &str, threshold: &str) -> Result<(), i32> {
  let threshold_value = match threshold.parse::<f64>() {
    Ok(v) => v,
    _ => panic!("Invalid threshold value: {threshold}"),
  };

  let docs = reader::read_file_as_yml(filepath);
  let doc = &docs[0];
  let items = doc.get("baseline").and_then(|v| v.as_sequence()).unwrap_or_else(|| panic!("Report file '{filepath}' does not contain a 'baseline' sequence"));
  let mut slow_counter = 0;

  for report in list_reports {
    for (i, report_item) in report.iter().enumerate() {
      let recorded_duration = items[i].get("duration").and_then(|v| v.as_f64()).unwrap();
      let delta_ms = report_item.duration - recorded_duration;

      if delta_ms > threshold_value {
        println!("{:width$} is {}{} slower than before", report_item.name.green(), delta_ms.round().to_string().red(), "ms".red(), width = 25);

        slow_counter += 1;
      }
    }
  }

  if slow_counter == 0 {
    Ok(())
  } else {
    Err(slow_counter)
  }
}
