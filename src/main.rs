pub mod parser;

use std::env;
use std::fs;

use crate::parser::parse;

fn main() {
    let args: Vec<String> = env::args().collect();
    let src_path = &args[1];
    
    if let Ok(code) = fs::read_to_string(src_path) {
        let tree = parse(&code);
    } else {
        println!("couldn't open the file");
    }
}