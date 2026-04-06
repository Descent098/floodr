//! File writing utilities.
//!
//! Minimal logic to write output (such as CSV dumps or metrics) to local files.

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

/// Synchronously writes a string to a file at the specified path.
///
/// If the file already exists, it will be overwritten. If it doesn't exist, 
/// it will be created along with any necessary parent directories.
///
/// # Arguments
///
/// - `filepath` (`&str`) - The path to the file to be written.
/// - `content` (`String`) - The string content to write to the file.
///
/// # Panics
///
/// - Panics if the file cannot be created or written to.
///
/// # Examples
///
/// ```rust,ignore
/// use floodr::writer::write_file;
/// write_file("report.txt", "data".to_string());
/// ```
pub fn write_file(filepath: &str, content: String) {
  let path = Path::new(filepath);
  let display = path.display();

  let mut file = match File::create(path) {
    Err(why) => panic!("couldn't create {display}: {why:?}"),
    Ok(file) => file,
  };

  if let Err(why) = file.write_all(content.as_bytes()) {
    panic!("couldn't write to {display}: {why:?}");
  }
}
