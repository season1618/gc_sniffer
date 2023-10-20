pub mod error;
pub mod parser;
pub mod metrics;

use std::env;
use std::fs;
use std::path;

use crate::error::AnalysisError;
use crate::parser::{parse, dump_tree, dump_attr};
use crate::metrics::{dump_metrics};

fn main() {
    let args: Vec<String> = env::args().collect();
    let path_str = &args[1];

    if let Err(err) = analyze_dirs(&path::Path::new(path_str)) {
        println!("{}", err);
    }
}

fn analyze_dirs(path: &path::Path) -> Result<(), AnalysisError> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            analyze_dirs(&entry.path())?;
        }
    } else if let Some(ext) = path.extension() {
        if ext.to_str() == Some("java") {
            let code = fs::read_to_string(path)?;
            let tree = parse(&code)?;
            let mut cursor = tree.walk();
            dump_attr(&mut cursor, code.as_bytes(), 0);
            dump_metrics(&mut cursor, code.as_bytes());
        }
    }
    Ok::<(), AnalysisError>(())
}