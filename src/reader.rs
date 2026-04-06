//! File reading and parsing utilities.
//!
//! Processes YAML benchmark files, parsing single and multi-document configurations,
//! as well as reading and decoding external CSV dependencies.

use serde_yaml::{Mapping, Value};
use std::fs::File;
use std::io::{BufReader, prelude::*};
use std::path::Path;
use std::process;
use colored::*;

/// Reads the entire contents of a file into a string.
///
/// # Arguments
///
/// - `filepath` (`&str`) - The path to the file to be read.
///
/// # Returns
///
/// - `String` - The contents of the file.
///
/// # Panics
///
/// - Panics if the file cannot be opened or read.
///
/// # Examples
///
/// ```rust,ignore
/// let content = read_file("test.txt");
/// ```
pub fn read_file(filepath: &str) -> String {
  // Create a path to the desired file
  let path = Path::new(filepath);
  let display = path.display();

  // Open the path in read-only mode, returns `io::Result<File>`
  let mut file = match File::open(path) {
      Ok(file) => file,
      Err(why) => {
          let formatted_error = format!("Couldn't open {}:\n\tReason: {}", display, why).red().bold();
          eprintln!("{}\n\nCheck the filename ({}), permissions, and try again\n\nUsage: \n\tfloodr <filename>.yml", formatted_error, filepath.to_string().cyan().bold());
          process::exit(1)
      }
  };

  // Read the file contents into a string, returns `io::Result<usize>`
  let mut content = String::new();
  if let Err(why) = file.read_to_string(&mut content) {
          let formatted_error = format!("Couldn't Read {}:\n\tReason: {}", display, why).red().bold();
          eprintln!("{}\n\nCheck permissions, and try again\n\nUsage: \n\tfloodr <filename>.yml", formatted_error);
          process::exit(1)
      }
  content
}

/// Internal utility to parse YAML content, supporting multi-document documents.
///
/// Serde YAML doesn't natively support multi-document splitting, so this manually
/// splits by the `---` separator and cleans up each document part.
///
/// # Arguments
///
/// - `content` (`&str`) - The raw YAML string.
///
/// # Returns
///
/// - `Vec<Value>` - A vector of parsed YAML documents.
fn parse_yaml_content(content: &str) -> Vec<Value> {
  // serde_yaml doesn't support multiple documents natively, so we split by "---\n" and parse each
  let mut docs = Vec::new();
  let trimmed_content = content.trim();

  // Handle multi-document YAML (separated by "---\n")
  if trimmed_content.contains("\n---\n") || (trimmed_content.starts_with("---\n") && trimmed_content.matches("---\n").count() > 1) {
    let parts: Vec<&str> = trimmed_content.split("---\n").collect();
    for doc_str in parts {
      let trimmed = doc_str.trim();
      // Skip empty parts and parts that are only comments
      if !trimmed.is_empty() && !trimmed.chars().all(|c| c == '#' || c.is_whitespace() || c == '\n') {
        match serde_yaml::from_str::<Value>(trimmed) {
          Ok(doc) => {
            // Skip Null documents (which can result from comments-only content)
            if !matches!(doc, Value::Null) {
              docs.push(doc);
            }
          }
          Err(e) => {
            eprintln!("Error parsing YAML document: {e}");
            panic!("Failed to parse YAML: {e}");
          }
        }
      }
    }
  }

  // If no documents were found (empty file or no "---"), try parsing the whole content
  if docs.is_empty() {
    // Remove leading "---\n" if present for single-document files
    let content_to_parse = trimmed_content.strip_prefix("---\n").unwrap_or(trimmed_content);
    match serde_yaml::from_str::<Value>(content_to_parse.trim()) {
      Ok(doc) => {
        if !matches!(doc, Value::Null) {
          docs.push(doc);
        }
      }
      Err(e) => {
        eprintln!("Error parsing YAML content: {e}");
        panic!("Failed to parse YAML: {e}");
      }
    }
  }

  // If still empty, return a single Null document to maintain compatibility
  if docs.is_empty() {
    docs.push(Value::Null);
  }

  docs
}

/// Reads a file and parses its contents as one or more YAML documents.
///
/// # Arguments
///
/// - `filepath` (`&str`) - The path to the YAML file.
///
/// # Returns
///
/// - `Vec<Value>` - A vector containing the parsed YAML documents.
///
/// # Examples
///
/// ```rust,ignore
/// let docs = read_file_as_yml("benchmark.yml");
/// ```
pub fn read_file_as_yml(filepath: &str) -> Vec<Value> {
  let content = read_file(filepath);
  parse_yaml_content(&content)
}

/// Utility for testing: parses a string directly into YAML documents.
///
/// # Arguments
///
/// - `content` (`&str`) - The YAML string.
///
/// # Returns
///
/// - `Vec<Value>` - The parsed documents.
#[cfg(test)]
pub fn read_file_as_yml_from_str(content: &str) -> Vec<Value> {
  parse_yaml_content(content)
}

/// Accesses a specific property within a YAML document or the document itself as a sequence.
///
/// # Arguments
///
/// - `doc` (`&'a Value`) - The YAML document to access.
/// - `accessor` (`Option<&str>`) - Optional key to access (e.g., "plan").
///
/// # Returns
///
/// - `&'a Vec<Value>` - A reference to the underlying sequence of values.
///
/// # Panics
///
/// - Panics if the accessor is provided but missing in the document.
/// - Panics if the resulting value is not a YAML sequence (array).
pub fn read_yaml_doc_accessor<'a>(doc: &'a Value, accessor: Option<&str>) -> &'a Vec<Value> {
  if let Some(accessor_id) = accessor {
    match doc.get(accessor_id).and_then(|v| v.as_sequence()) {
      Some(items) => items,
      None => {
        println!("Node missing on config: {accessor_id}");
        println!("Exiting.");
        std::process::exit(1)
      }
    }
  } else {
    doc.as_sequence().unwrap_or_else(|| panic!("Expected document to be a sequence, got: {doc:?}"))
  }
}

/// Reads a text file line-by-line and returns the lines as a vector of YAML strings.
///
/// # Arguments
///
/// - `filepath` (`&str`) - The path to the text file.
///
/// # Returns
///
/// - `Vec<Value>` - A vector of YAML strings, one per line.
pub fn read_file_as_yml_array(filepath: &str) -> Vec<Value> {
  let path = Path::new(filepath);
  let display = path.display();

  let file = match File::open(path) {
    Err(why) => panic!("couldn't open {display}: {why}"),
    Ok(file) => file,
  };

  let reader = BufReader::new(file);
  let mut items = Vec::new();
  for line in reader.lines() {
    match line {
      Ok(text) => {
        items.push(Value::String(text));
      }
      Err(e) => println!("error parsing line: {e:?}"),
    }
  }

  items
}

/// Reads a CSV file and converts its rows into a vector of YAML mappings.
///
/// The first row is expected to be headers, and it is used as keys for each
/// subsequent row's map representation.
///
/// # Arguments
///
/// - `filepath` (`&str`) - The path to the CSV file.
/// - `quote` (`u8`) - The character used for quoting CSV fields (e.g., `b'"'`).
///
/// # Returns
///
/// - `Vec<Value>` - A vector of YAML mappings representing each row.
pub fn read_csv_file_as_yml(filepath: &str, quote: u8) -> Vec<Value> {
  // TODO: Try to split this fn into two
  // Create a path to the desired file
  let path = Path::new(filepath);
  let display = path.display();

  // Open the path in read-only mode, returns `io::Result<File>`
  let file = match File::open(path) {
    Err(why) => panic!("couldn't open {display}: {why}"),
    Ok(file) => file,
  };

  let mut rdr = csv::ReaderBuilder::new().has_headers(true).quote(quote).from_reader(file);

  let mut items = Vec::new();

  let headers = match rdr.headers() {
    Err(why) => panic!("error parsing header: {why:?}"),
    Ok(h) => h.clone(),
  };

  for result in rdr.records() {
    match result {
      Ok(record) => {
        let mut mapping = Mapping::new();

        for (i, header) in headers.iter().enumerate() {
          let item_key = Value::String(header.to_string());
          let item_value = Value::String(record.get(i).unwrap().to_string());

          mapping.insert(item_key, item_value);
        }

        items.push(Value::Mapping(mapping));
      }
      Err(e) => println!("error parsing header: {e:?}"),
    }
  }

  items
}
