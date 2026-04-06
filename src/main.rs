//! Main entry point for the Floodr CLI application.
//!
//! Defers execution to the library crate `floodr`.

/// The main entry point calling `floodr::main()`.
///
/// This function simply delegates to the library's `main` function to handle
/// the application logic, parsing, and execution.
///
/// # Examples
///
/// ```bash
/// # Run the application
/// cargo run -- --benchmark my_test.yml
/// ```
fn main() {
  floodr::main();
}
