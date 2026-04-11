//! Main entry point for the Floodr CLI application.
//!
//! Defers execution to the library crate `floodr`.

use floodr::engine::benchmark;
use floodr::parsing::tags;
use std::process;
use colored::control;
use clap::{crate_version, Parser};

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
#[derive(Parser, Debug)]
#[command(
  name = "drill",
  version = crate_version!(),
  about = "A configurable, simple rust-based HTTP load testing system",
  long_about = None,
)]
struct Cli {
  /// Benchmark file to run
  #[arg(default_value = "benchmark.yaml")]
  benchmark: String,

  /// Shows request statistics
  #[arg(short = 's', long = "stats", conflicts_with = "compare")]
  stats: bool,

  /// Sets a report file
  #[arg(short = 'r', long = "report", conflicts_with = "compare")]
  report: Option<String>,

  /// Sets a compare file
  #[arg(short = 'c', long = "compare", conflicts_with = "report")]
  compare: Option<String>,

  /// Sets a threshold value in ms amongst the compared file
  #[arg(short = 't', long = "threshold", conflicts_with = "report", requires = "compare")]
  threshold: Option<String>,

  /// Do not panic if an interpolation is not present. (Not recommended)
  #[arg(long = "relaxed-interpolations")]
  relaxed_interpolations: bool,

  /// Disables SSL certification check. (Not recommended)
  #[arg(long = "no-check-certificate")]
  no_check_certificate: bool,

  /// Tags to include
  #[arg(long = "tags")]
  tags: Option<String>,

  /// Tags to exclude
  #[arg(long = "skip-tags")]
  skip_tags: Option<String>,

  /// List all benchmark tags
  #[arg(long = "list-tags", conflicts_with_all = ["tags", "skip_tags"])]
  list_tags: bool,

  /// List benchmark tasks (executes --tags/--skip-tags filter)
  #[arg(long = "list-tasks")]
  list_tasks: bool,

  /// Disables output
  #[arg(short = 'q', long = "quiet")]
  quiet: bool,

  /// Set timeout in seconds for a request
  #[arg(long = "request-timeout")]
  request_timeout: Option<String>,

  /// Toggle verbose output
  #[arg(short = 'v', long = "verbose")]
  verbose: bool,
}

impl Cli {
  fn run(self) -> process::ExitCode {

    #[cfg(windows)]
    let _ = control::set_virtual_terminal(true);

    if self.list_tags {
      tags::list_benchmark_file_tags(&self.benchmark);
      process::exit(0);
    }

    let tags = tags::Tags::new(self.tags.as_deref(), self.skip_tags.as_deref());

    if self.list_tasks {
      tags::list_benchmark_file_tasks(&self.benchmark, &tags);
      process::exit(0);
    }

    let benchmark_result = benchmark::execute(
      &self.benchmark,
      self.report.as_deref(),
      self.relaxed_interpolations,
      self.no_check_certificate,
      self.quiet,
      self.request_timeout.as_deref(),
      self.verbose,
      &tags,
    );

    let list_reports = benchmark_result.reports;
    let duration = benchmark_result.duration;

    floodr::show_stats(&list_reports, self.stats, duration);
    floodr::compare_benchmark(
      &list_reports,
      self.compare.as_deref(),
      self.threshold.as_deref(),
    );

    return process::ExitCode::SUCCESS;
  }
}

fn main() -> process::ExitCode {
  return Cli::parse().run();
}

