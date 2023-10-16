pub mod parser;
pub mod metrics;

use std::env;
use std::fs;

use crate::parser::{parse, dump_tree, dump_attr};
use crate::metrics::{metrics};

fn main() {
    let args: Vec<String> = env::args().collect();
    let src_path = &args[1];
    
    if let Ok(code) = fs::read_to_string(src_path) {
        if let Ok(tree) = parse(&code) {
            let mut cursor = tree.walk();
            // dump_tree(&mut cursor, 0);
            dump_attr(&mut cursor, code.as_bytes(), 0);
            for class in metrics(&mut cursor, code.as_bytes()) {
                class.dump_metrics();
            }
        }
    } else {
        println!("couldn't open the file");
    }
}