//! Parsing and configuration utilities for Floodr.
//!
//! This module contains all logic related to reading, parsing, and interpreting
//! benchmark files, configuration, tags, and variable interpolation.
//! 
//! ## Example
//! 
//! ### Execute from file
//! 
//! ```rust,ignore
//! use floodr::parsing::config::Config;
//! use floodr::engine::benchmark;
//! 
//! let config = Config::new("benchmark.yml", false, false, false, 1000, false, None, None);
//! let tags = &Tags::new(None, None);
//! 
//! benchmark::execute(
//!     "benchmark.yml".as_ref(), 
//!     None, 
//!     false, 
//!     config.no_check_certificate, 
//!     config.quiet, 
//!     Some(&config.timeout.to_string()), 
//!     config.verbose, 
//!     config.exec_terminal.as_deref(), 
//!     &tags, 
//!     None
//! );
//! ```
//! 
//! ### Manually Parse and execute plan with no Concurrency
//! 
//! ```rust, ignore
//! use floodr::parsing::config::Config;
//! use floodr::engine::benchmark;
//! use serde_yaml;
//! 
//! let config = Config::new("benchmark.yml", false, false, false, 1000, false, None, None);
//! 
//! let plan_data = r#"
//! - name: Fetch account
//!   request:
//!     url: /"#;
//!     
//! let plan_items = serde_yaml::from_str(plan_data).expect("Failed to parse");
//! println!("{:#?}", plan_items);
//!
//! let benchmark_plan = floodr::parsing::comparison_loader::load_from_items(plan_items);
//! benchmark::execute_from_plan(
//!     benchmark_plan,
//!     "http://localhost:4896".to_string(),
//!     config.relaxed_interpolations,
//!     config.no_check_certificate, 
//!     config.quiet,
//!     config.timeout,
//!     config.verbose,
//!     config.exec_terminal
//! );
//! ```

pub mod benchmark;
