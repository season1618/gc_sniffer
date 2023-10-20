use tree_sitter::{Parser, Language, Tree, TreeCursor};
use crate::error::AnalysisError::{self, ParseError};

extern "C" { fn tree_sitter_java() -> Language; }

pub fn parse(code: &String) -> Result<Tree, AnalysisError> {
    let mut parser = Parser::new();
    parser.set_language(unsafe { tree_sitter_java() }).unwrap();
    parser.parse(&code.clone(), None).ok_or(ParseError)
}

pub fn dump_tree(cursor: &mut TreeCursor, indent: usize) {
    println!("{}{}", " ".repeat(indent), cursor.node().kind());

    if cursor.goto_first_child() {
        dump_tree(cursor, indent + 4);
        while cursor.goto_next_sibling() {
            dump_tree(cursor, indent + 4);
        }
        cursor.goto_parent();
    }
}

pub fn dump_attr(cursor: &mut TreeCursor, code: &[u8], indent: usize) {
    match cursor.node().kind() {
        "class_declaration" => {
            let name = cursor
                .node()
                .child_by_field_name("name").unwrap()
                .utf8_text(code).unwrap();
            
            println!("{}class {}", " ".repeat(indent), name);
        },
        "field_declaration" => {
            let type_name = cursor
                .node()
                .child_by_field_name("type").unwrap()
                .utf8_text(code).unwrap();

            cursor.goto_first_child();
            loop {
                if cursor.field_name() == Some("declarator") {
                    let name = cursor
                        .node()
                        .child_by_field_name("name").unwrap()
                        .utf8_text(code).unwrap().to_string();

                    println!("{}{}: {}", " ".repeat(indent), name, type_name);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        },
        "method_declaration" => {
            let type_name = cursor
                .node()
                .child_by_field_name("type").unwrap()
                .utf8_text(code).unwrap();
            
            let name = cursor
                .node()
                .child_by_field_name("name").unwrap()
                .utf8_text(code).unwrap();
            
            println!("{}{}(): {}", " ".repeat(indent), name, type_name);
        },
        _ => {},
    }

    if cursor.goto_first_child() {
        dump_attr(cursor, code, indent + 4);
        while cursor.goto_next_sibling() {
            dump_attr(cursor, code, indent + 4);
        }
        cursor.goto_parent();
    }
}