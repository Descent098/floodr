//! Main entry point for the Floodr CLI application.
//!
//! Defers execution to the library crate `floodr`.

use std::process;
use colored::control;

/// The main entry point calling `floodr::main()`.
///
/// It parses command line arguments and initiates the benchmark execution based
/// on the provided configuration.
///
/// # Examples
///
/// ```bash
/// # Run the application
/// cargo run -- <file>.yml
/// ```
fn main() {
  // TODO: update to builder pattern to make the type conversions cleaner
  let matches = floodr::app_args();
  let benchmark_file = matches.get_one::<String>("benchmark").unwrap().as_str();
  let report_path_option = Some(matches.get_one::<String>("report").unwrap().as_str());
  let stats_option = matches.contains_id("stats");
  let compare_path_option = Some(matches.get_one::<String>("compare").unwrap().as_str());
  let threshold_option = Some(matches.get_one::<String>("threshold").unwrap().as_str());
  let no_check_certificate = matches.contains_id("no-check-certificate");
  let relaxed_interpolations = matches.contains_id("relaxed-interpolations");
  let quiet = matches.contains_id("quiet");
  let nanosec = matches.contains_id("nanosec");
  let timeout = Some(matches.get_one::<String>("timeout").unwrap().as_str());
  let verbose = matches.contains_id("verbose");
  let tags_option = Some(matches.get_one::<String>("tags").unwrap().as_str());
  let skip_tags_option = Some(matches.get_one::<String>("skip-tags").unwrap().as_str());
  let list_tags = matches.contains_id("list-tags");
  let list_tasks = matches.contains_id("list-tasks");

  #[cfg(windows)]
  let _ = control::set_virtual_terminal(true);

  if list_tags {
    floodr::parsing::tags::list_benchmark_file_tags(benchmark_file);
    process::exit(0);
  };

  let tags = floodr::parsing::tags::Tags::new(tags_option, skip_tags_option);

  if list_tasks {
    floodr::parsing::tags::list_benchmark_file_tasks(benchmark_file, &tags);
    process::exit(0);
  };

  let benchmark_result = floodr::engine::benchmark::execute(benchmark_file, report_path_option, relaxed_interpolations, no_check_certificate, quiet, nanosec, timeout, verbose, &tags);
  let list_reports = benchmark_result.reports;
  let duration = benchmark_result.duration;

  floodr::show_stats(&list_reports, stats_option, nanosec, duration);
  floodr::compare_benchmark(&list_reports, compare_path_option, threshold_option);

  process::exit(0)
}
