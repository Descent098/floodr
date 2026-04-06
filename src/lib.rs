//! Floodr library crate
//!
//! Provides the core execution logic for Floodr

pub mod actions;
pub mod benchmark;
mod checker;
pub mod config;
pub mod expandable;
mod interpolator;
mod reader;
pub mod tags;
mod writer;

use crate::actions::Report;
use clap::crate_version;
use clap::{App, Arg};
use colored::*;
use hdrhistogram::Histogram;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use std::process;


/// Configure and parse command line arguments for the application.
///
/// This uses the `clap` crate to define the CLI interface, including options for
/// benchmarks, reports, statistics, and tags.
///
/// # Returns
///
/// - `clap::ArgMatches<'a>` - The parsed command line arguments.
pub fn app_args<'a>() -> clap::ArgMatches<'a> {
  App::new("floodr")
    .version(crate_version!())
    .about("HTTP load testing application written in Rust inspired by Ansible syntax")
    .arg(
      Arg::with_name("benchmark")
        .help("Sets the benchmark file")
        .index(1)
        .required(false)
        .default_value("benchmark.yaml")
    )
    .arg(Arg::with_name("stats").short("s").long("stats").help("Shows request statistics").takes_value(false).conflicts_with("compare"))
    .arg(Arg::with_name("report").short("r").long("report").help("Sets a report file").takes_value(true).conflicts_with("compare"))
    .arg(Arg::with_name("compare").short("c").long("compare").help("Sets a compare file").takes_value(true).conflicts_with("report"))
    .arg(Arg::with_name("threshold").short("t").long("threshold").help("Sets a threshold value in ms amongst the compared file").takes_value(true).conflicts_with("report"))
    .arg(Arg::with_name("relaxed-interpolations").long("relaxed-interpolations").help("Do not panic if an interpolation is not present. (Not recommended)").takes_value(false))
    .arg(Arg::with_name("no-check-certificate").long("no-check-certificate").help("Disables SSL certification check. (Not recommended)").takes_value(false))
    .arg(Arg::with_name("tags").long("tags").help("Tags to include").takes_value(true))
    .arg(Arg::with_name("skip-tags").long("skip-tags").help("Tags to exclude").takes_value(true))
    .arg(Arg::with_name("list-tags").long("list-tags").help("List all benchmark tags").takes_value(false).conflicts_with_all(&["tags", "skip-tags"]))
    .arg(Arg::with_name("list-tasks").long("list-tasks").help("List benchmark tasks (executes --tags/--skip-tags filter)").takes_value(false))
    .arg(Arg::with_name("quiet").short("q").long("quiet").help("Disables output").takes_value(false))
    .arg(Arg::with_name("timeout").short("o").long("timeout").help("Set timeout in seconds for all requests").takes_value(true))
    .arg(Arg::with_name("nanosec").short("n").long("nanosec").help("Shows statistics in nanoseconds").takes_value(false))
    .arg(Arg::with_name("verbose").short("v").long("verbose").help("Toggle verbose output").takes_value(false))
    .get_matches()
}

/// Holds details about the results of a floodr execution.
///
/// This struct captures aggregate metrics across all requests in a benchmark.
///
/// # Fields
///
/// - `total_requests` (`usize`) - The total number of requests run after all actions completed in the plan.
/// - `successful_requests` (`usize`) - The total number of successful requests (HTTP 2xx).
/// - `failed_requests` (`usize`) - The total number of failed requests.
/// - `hist` (`Histogram<u64>`) - The histogram of response durations in microseconds.
pub struct FloodrStats {
  pub total_requests: usize,      // The total number of requests run after all actions completed in the plan
  pub successful_requests: usize, // The total number of successful requests run after all actions completed in the plan
  pub failed_requests: usize,     // The total number of failed requests run after all actions completed in the plan
  pub hist: Histogram<u64>,       // The histogram of data about the run
}

impl FloodrStats {
  /// Returns the mean duration of the requests in milliseconds.
  ///
  /// # Returns
  ///
  /// - `f64` - The mean duration.
  pub fn mean_duration(&self) -> f64 {
    self.hist.mean() / 1_000.0
  }
  /// Returns the median (50th percentile) duration of the requests in milliseconds.
  ///
  /// # Returns
  ///
  /// - `f64` - The median duration.
  pub fn median_duration(&self) -> f64 {
    self.hist.value_at_quantile(0.5) as f64 / 1_000.0
  }
  /// Returns the sample standard deviation of durations in milliseconds.
  ///
  /// # Returns
  ///
  /// - `f64` - The standard deviation.
  pub fn stdev_duration(&self) -> f64 {
    self.hist.stdev() / 1_000.0
  }
  /// Returns the duration at a given percentile in milliseconds.
  ///
  /// # Arguments
  ///
  /// - `quantile` (`f64`) - The quantile (0.0 to 1.0).
  ///
  /// # Returns
  ///
  /// - `f64` - The value at the specified quantile.
  pub fn value_at_quantile(&self, quantile: f64) -> f64 {
    self.hist.value_at_quantile(quantile) as f64 / 1_000.0
  }
}

/// Computes aggregate statistics for a set of request reports.
///
/// # Arguments
///
/// - `sub_reports` (`&[Report]`) - A slice of request reports to analyze.
///
/// # Returns
///
/// - `FloodrStats` - The computed statistics including totals and duration histogram.
///
/// # Examples
///
/// ```rust,ignore
/// use floodr::actions::Report;
/// let reports = vec![Report { name: "test".to_string(), duration: 100.0, status: 200 }];
/// let stats = compute_stats(&reports);
/// ```
pub fn compute_stats(sub_reports: &[Report]) -> FloodrStats {
  let max_duration_us = 60 * 60 * 1_000 * 1_000; // 1 hour in microseconds
  let mut hist = Histogram::<u64>::new_with_bounds(1, max_duration_us, 2).unwrap();
  let mut group_by_status = HashMap::new();

  for req in sub_reports {
    group_by_status.entry(req.status / 100).or_insert_with(Vec::new).push(req);
  }

  for r in sub_reports.iter() {
    let duration_us = (r.duration * 1_000.0) as u64;

    if let Err(err) = hist.record(duration_us) {
      eprintln!("warning: failed to record histogram value for request '{}' (duration={} ms, duration_us={}): {}", r.name, r.duration, duration_us, err);
    }
  }

  let total_requests = sub_reports.len();
  let successful_requests = group_by_status.entry(2).or_insert_with(Vec::new).len();
  let failed_requests = total_requests - successful_requests;

  FloodrStats {
    total_requests,
    successful_requests,
    failed_requests,
    hist,
  }
}

/// Formats a time duration into a human-readable string.
///
/// # Arguments
///
/// - `tdiff` (`f64`) - The duration in milliseconds.
/// - `nanosec` (`bool`) - Whether to output the value in nanoseconds.
///
/// # Returns
///
/// - `String` - The formatted duration (e.g., "100ms" or "100000ns").
///
/// # Examples
///
/// ```rust,ignore
/// let time = format_time(100.5, false);
/// assert_eq!(time, "101ms");
/// ```
pub fn format_time(tdiff: f64, nanosec: bool) -> String {
  if nanosec {
    (1_000_000.0 * tdiff).round().to_string() + "ns"
  } else {
    tdiff.round().to_string() + "ms"
  }
}

/// Displays formatted statistics reports into the standard output.
///
/// # Arguments
///
/// - `list_reports` (`&[Vec<Report>]`) - A collection of report vectors (one per iteration).
/// - `stats_option` (`bool`) - Whether to actually display the statistics.
/// - `nanosec` (`bool`) - Whether to display durations in nanoseconds.
/// - `duration` (`f64`) - The total execution duration of the benchmark.
///
pub fn show_stats(list_reports: &[Vec<Report>], stats_option: bool, nanosec: bool, duration: f64) {
  if !stats_option {
    return;
  }

  let mut group_by_name = LinkedHashMap::new();

  for req in list_reports.concat() {
    group_by_name.entry(req.name.clone()).or_insert_with(Vec::new).push(req);
  }

  // compute stats per name
  for (name, reports) in group_by_name {
    let substats = compute_stats(&reports);
    println!();
    println!("{:width$} {:width2$} {}", name.green(), "Total requests".yellow(), substats.total_requests.to_string().purple(), width = 25, width2 = 25);
    println!("{:width$} {:width2$} {}", name.green(), "Successful requests".yellow(), substats.successful_requests.to_string().purple(), width = 25, width2 = 25);
    println!("{:width$} {:width2$} {}", name.green(), "Failed requests".yellow(), substats.failed_requests.to_string().purple(), width = 25, width2 = 25);
    println!("{:width$} {:width2$} {}", name.green(), "Median time per request".yellow(), format_time(substats.median_duration(), nanosec).purple(), width = 25, width2 = 25);
    println!("{:width$} {:width2$} {}", name.green(), "Average time per request".yellow(), format_time(substats.mean_duration(), nanosec).purple(), width = 25, width2 = 25);
    println!("{:width$} {:width2$} {}", name.green(), "Sample standard deviation".yellow(), format_time(substats.stdev_duration(), nanosec).purple(), width = 25, width2 = 25);
    println!("{:width$} {:width2$} {}", name.green(), "99.0'th percentile".yellow(), format_time(substats.value_at_quantile(0.99), nanosec).purple(), width = 25, width2 = 25);
    println!("{:width$} {:width2$} {}", name.green(), "99.5'th percentile".yellow(), format_time(substats.value_at_quantile(0.995), nanosec).purple(), width = 25, width2 = 25);
    println!("{:width$} {:width2$} {}", name.green(), "99.9'th percentile".yellow(), format_time(substats.value_at_quantile(0.999), nanosec).purple(), width = 25, width2 = 25);
  }

  // compute global stats
  let allreports = list_reports.concat();
  let global_stats = compute_stats(&allreports);
  let requests_per_second = global_stats.total_requests as f64 / duration;

  println!();
  println!("{:width2$} {} {}", "Time taken for tests".yellow(), format!("{duration:.1}").purple(), "seconds".purple(), width2 = 25);
  println!("{:width2$} {}", "Total requests".yellow(), global_stats.total_requests.to_string().purple(), width2 = 25);
  println!("{:width2$} {}", "Successful requests".yellow(), global_stats.successful_requests.to_string().purple(), width2 = 25);
  println!("{:width2$} {}", "Failed requests".yellow(), global_stats.failed_requests.to_string().purple(), width2 = 25);
  println!("{:width2$} {} {}", "Requests per second".yellow(), format!("{requests_per_second:.2}").purple(), "[#/sec]".purple(), width2 = 25);
  println!("{:width2$} {}", "Median time per request".yellow(), format_time(global_stats.median_duration(), nanosec).purple(), width2 = 25);
  println!("{:width2$} {}", "Average time per request".yellow(), format_time(global_stats.mean_duration(), nanosec).purple(), width2 = 25);
  println!("{:width2$} {}", "Sample standard deviation".yellow(), format_time(global_stats.stdev_duration(), nanosec).purple(), width2 = 25);
  println!("{:width2$} {}", "99.0'th percentile".yellow(), format_time(global_stats.value_at_quantile(0.99), nanosec).purple(), width2 = 25);
  println!("{:width2$} {}", "99.5'th percentile".yellow(), format_time(global_stats.value_at_quantile(0.995), nanosec).purple(), width2 = 25);
  println!("{:width2$} {}", "99.9'th percentile".yellow(), format_time(global_stats.value_at_quantile(0.999), nanosec).purple(), width2 = 25);
}

/// Compares current execution metrics against a previous benchmark report and checks for performance regressions.
///
/// # Arguments
///
/// - `list_reports` (`&[Vec<Report>]`) - The current benchmark results.
/// - `compare_path_option` (`Option<&str>`) - Path to the baseline report file.
/// - `threshold_option` (`Option<&str>`) - The threshold in milliseconds allowed for performance drops.
///
/// # Panics
///
/// - Panics if a `compare_path` is provided but no `threshold` is specified.
///
/// # Examples
///
/// ```rust,ignore
/// compare_benchmark(&reports, Some("baseline.yml"), Some("50"));
/// ```
pub fn compare_benchmark(list_reports: &[Vec<Report>], compare_path_option: Option<&str>, threshold_option: Option<&str>) {
  if let Some(compare_path) = compare_path_option {
    if let Some(threshold) = threshold_option {
      let compare_result = checker::compare(list_reports, compare_path, threshold);

      match compare_result {
        Ok(_) => process::exit(0),
        Err(_) => process::exit(1),
      }
    } else {
      panic!("Threshold needed!");
    }
  }
}
