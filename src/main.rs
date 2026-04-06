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
/// cargo run -- --benchmark my_test.yml
/// ```
fn main() {
  let matches = floodr::app_args();
  let benchmark_file = matches.value_of("benchmark").unwrap();
  let report_path_option = matches.value_of("report");
  let stats_option = matches.is_present("stats");
  let compare_path_option = matches.value_of("compare");
  let threshold_option = matches.value_of("threshold");
  let no_check_certificate = matches.is_present("no-check-certificate");
  let relaxed_interpolations = matches.is_present("relaxed-interpolations");
  let quiet = matches.is_present("quiet");
  let nanosec = matches.is_present("nanosec");
  let timeout = matches.value_of("timeout");
  let verbose = matches.is_present("verbose");
  let tags_option = matches.value_of("tags");
  let skip_tags_option = matches.value_of("skip-tags");
  let list_tags = matches.is_present("list-tags");
  let list_tasks = matches.is_present("list-tasks");

  #[cfg(windows)]
  let _ = control::set_virtual_terminal(true);

  if list_tags {
    floodr::tags::list_benchmark_file_tags(benchmark_file);
    process::exit(0);
  };

  let tags = floodr::tags::Tags::new(tags_option, skip_tags_option);

  if list_tasks {
    floodr::tags::list_benchmark_file_tasks(benchmark_file, &tags);
    process::exit(0);
  };

  let benchmark_result = floodr::benchmark::execute(benchmark_file, report_path_option, relaxed_interpolations, no_check_certificate, quiet, nanosec, timeout, verbose, &tags);
  let list_reports = benchmark_result.reports;
  let duration = benchmark_result.duration;

  floodr::show_stats(&list_reports, stats_option, nanosec, duration);
  floodr::compare_benchmark(&list_reports, compare_path_option, threshold_option);

  process::exit(0)
}
