//! Main entry point for the Floodr CLI application.
//!
//! Defers execution to the library crate `floodr`.

use clap::{Parser, Subcommand, crate_version};
use floodr::engine::benchmark;
use floodr::parsing::tags;
use std::process;

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
  name = "floodr",
  version = crate_version!(),
  about = "A configurable, simple rust-based HTTP load testing system",
  long_about = None,
)]
struct Cli {
  /// Benchmark file to run
  #[arg(default_value = "benchmark.yml")]
  benchmark: String,

  /// Shows request statistics
  #[arg(short = 's', long = "stats")]
  stats: bool,

  /// Subcommand to execute
  #[command(subcommand)]
  command: Option<Commands>,

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
  #[arg(short = 'q', long = "quiet", conflicts_with="verbose")]
  quiet: bool,

  /// Set timeout in seconds for a request
  #[arg(long = "request-timeout")]
  request_timeout: Option<String>,

  /// Toggle verbose output
  #[arg(short = 'v', long = "verbose", conflicts_with="quiet")]
  verbose: bool,

  /// Set the terminal to run exec commands with
  #[arg(long = "exec-terminal")]
  exec_terminal: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Compares current execution metrics against a previous benchmark report
  Compare {
    /// Baseline report file to compare against
    report_file: String,
    /// Threshold value in milliseconds
    threshold: String,
  },
  /// Writes benchmark results to a report file
  Reports {
    /// Report file to write
    report_file: String,
  },
}

#[allow(unused_imports)]
impl Cli {
  fn run(self) -> process::ExitCode {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    if self.list_tags {
      tags::list_benchmark_file_tags(&self.benchmark);
      return process::ExitCode::SUCCESS;
    }

    let tags = tags::Tags::new(self.tags.as_deref(), self.skip_tags.as_deref());

    if self.list_tasks {
      tags::list_benchmark_file_tasks(&self.benchmark, &tags);
      return process::ExitCode::SUCCESS;
    }

    let mut base_override = None;
    if let Some(Commands::Compare {
      ref report_file,
      ..
    }) = self.command
    {
      base_override = Some(floodr::parsing::checker::get_base(report_file));
    }

    let report_path = match &self.command {
      Some(Commands::Reports { report_file }) => Some(report_file.as_str()),
      _ => None,
    };

    let benchmark_result = if let Some(Commands::Compare {
      ref report_file,
      ..
    }) = self.command
    {
      let (base, plan_items, _) = floodr::parsing::comparison_loader::load_report_data(report_file);
      let benchmark_plan = floodr::parsing::comparison_loader::load_from_items(plan_items);

      benchmark::execute_from_plan(benchmark_plan, base, false, self.no_check_certificate, self.quiet, self.request_timeout.as_deref().map_or(10, |t| t.parse().unwrap_or(10)), self.verbose, self.exec_terminal.clone())
    } else {
      benchmark::execute(&self.benchmark, report_path, false, self.no_check_certificate, self.quiet, self.request_timeout.as_deref(), self.verbose, self.exec_terminal.as_deref(), &tags, base_override)
    };

    let list_reports = benchmark_result.reports;
    let duration = benchmark_result.duration;

    floodr::show_stats(&list_reports, self.stats, duration);

    if let Some(Commands::Compare {
      report_file,
      threshold,
    }) = self.command
    {
      floodr::compare_benchmark(&list_reports, Some(&report_file), Some(&threshold));
    }

    process::ExitCode::SUCCESS
  }
}

fn main() -> process::ExitCode {
  Cli::parse().run()
}
