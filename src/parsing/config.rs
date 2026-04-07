//! Configuration module.
//!
//! Exposes and parses the properties necessary to adapt the behavior
//! of the benchmark runner (eg. concurrency, iterations, options).

use serde_yaml::Value;

use crate::engine::benchmark::Context;
use crate::parsing::interpolator;
use crate::parsing::reader;

const NITERATIONS: i64 = 1;
const NRAMPUP: i64 = 0;

/// Represents the configuration for the benchmark runner.
///
/// # Fields
///
/// - `base` (`String`) - The base URL for the benchmark
/// - `concurrency` (`i64`) - The number of concurrent requests
/// - `iterations` (`i64`) - The number of iterations to run
/// - `relaxed_interpolations` (`bool`) - Whether to allow relaxed interpolations
/// - `no_check_certificate` (`bool`) - Whether to skip certificate checking
/// - `rampup` (`i64`) - The rampup time for the benchmark
/// - `quiet` (`bool`) - Whether to run the benchmark in quiet mode
/// - `nanosec` (`bool`) - Whether to use nanosecond precision for timing
/// - `timeout` (`u64`) - The timeout for the benchmark
/// - `verbose` (`bool`) - Whether to run the benchmark in verbose mode
///
/// # Examples
///
/// ```rust,no_run
/// use floodr::parsing::config::Config;
///
/// let config = Config::new("config.yml", false, false, false, false, 1000, false);
/// ```
pub struct Config {
  pub base: String,
  pub concurrency: i64,
  pub iterations: i64,
  pub relaxed_interpolations: bool,
  pub no_check_certificate: bool,
  pub rampup: i64,
  pub quiet: bool,
  pub nanosec: bool,
  pub timeout: u64,
  pub verbose: bool,
}

impl Config {
  /// Creates a new `Config` instance by reading the benchmark file.
  ///
  /// This parses the YAML configuration and resolves placeholders for
  /// properties like iterations, concurrency, and rampup using an empty context.
  ///
  /// # Arguments
  ///
  /// - `path` (`&str`) - Path to the benchmark YAML file.
  /// - `relaxed_interpolations` (`bool`) - Whether missing interpolations cause panics.
  /// - `no_check_certificate` (`bool`) - Whether to disable SSL certificate checks.
  /// - `quiet` (`bool`) - Whether to minimize output.
  /// - `nanosec` (`bool`) - Whether to use nanosecond timing.
  /// - `timeout` (`u64`) - Request timeout in seconds.
  /// - `verbose` (`bool`) - Whether to enable verbose logging.
  ///
  /// # Returns
  ///
  /// - `Config` - The parsed and initialized configuration.
  ///
  /// # Panics
  ///
  /// - Panics if the concurrency is higher than the number of iterations.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// let config = Config::new("test.yml", false, false, false, false, 10, true);
  /// ```
  pub fn new(path: &str, relaxed_interpolations: bool, no_check_certificate: bool, quiet: bool, nanosec: bool, timeout: u64, verbose: bool) -> Config {
    let config_docs = reader::read_file_as_yml(path);
    let config_doc = &config_docs[0];

    let context: Context = Context::new();
    let interpolator = interpolator::Interpolator::new(&context);

    let iterations = read_i64_configuration(config_doc, &interpolator, "iterations", NITERATIONS);
    let concurrency = read_i64_configuration(config_doc, &interpolator, "concurrency", iterations);
    let rampup = read_i64_configuration(config_doc, &interpolator, "rampup", NRAMPUP);
    let base = read_str_configuration(config_doc, &interpolator, "base", "");

    if concurrency > iterations {
      panic!("The concurrency can not be higher than the number of iterations")
    }

    Config {
      base,
      concurrency,
      iterations,
      relaxed_interpolations,
      no_check_certificate,
      rampup,
      quiet,
      nanosec,
      timeout,
      verbose,
    }
  }
}

/// Reads a string configuration property from the YAML document.
///
/// # Arguments
///
/// - `config_doc` (`&Value`) - The YAML document value.
/// - `interpolator` (`&interpolator::Interpolator`) - An interpolator to resolve potential placeholders.
/// - `name` (`&str`) - The name of the property to read.
/// - `default` (`&str`) - The default value if the property is missing.
///
/// # Returns
///
/// - `String` - The resolved configuration string.
fn read_str_configuration(config_doc: &Value, interpolator: &interpolator::Interpolator, name: &str, default: &str) -> String {
  match config_doc.get(name).and_then(|v| v.as_str()) {
    Some(value) => {
      if value.contains('{') {
        interpolator.resolve(value, true)
      } else {
        value.to_owned()
      }
    }
    None => {
      if config_doc.get(name).and_then(|v| v.as_str()).is_some() {
        println!("Invalid {name} value!");
      }

      default.to_owned()
    }
  }
}

/// Reads an integer (i64) configuration property from the YAML document.
///
/// # Arguments
///
/// - `config_doc` (`&Value`) - The YAML document value.
/// - `interpolator` (`&interpolator::Interpolator`) - An interpolator to resolve potential placeholders.
/// - `name` (`&str`) - The name of the property to read.
/// - `default` (`i64`) - The default value if the property is missing or invalid.
///
/// # Returns
///
/// - `i64` - The resolved configuration integer.
fn read_i64_configuration(config_doc: &Value, interpolator: &interpolator::Interpolator, name: &str, default: i64) -> i64 {
  let value = if let Some(value) = config_doc.get(name).and_then(|v| v.as_i64()) {
    Some(value)
  } else if let Some(key) = config_doc.get(name).and_then(|v| v.as_str()) {
    interpolator.resolve(key, false).parse::<i64>().ok()
  } else {
    None
  };

  match value {
    Some(value) => {
      if value < 0 {
        println!("Invalid negative {name} value!");

        default
      } else {
        value
      }
    }
    None => {
      if config_doc.get(name).and_then(|v| v.as_str()).is_some() {
        println!("Invalid {name} value!");
      }

      default
    }
  }
}
