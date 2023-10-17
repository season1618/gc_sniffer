use tree_sitter::{Parser, Language, Tree, TreeCursor};

extern "C" { fn tree_sitter_java() -> Language; }

pub fn parse(code: &String) -> Result<Tree, &str> {
    let mut parser = Parser::new();
    parser.set_language(unsafe { tree_sitter_java() }).unwrap();
    parser.parse(&code.clone(), None).ok_or("failed to parse")
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

            let mut cursor2 = cursor.clone();
            let mut decl_node_list = cursor
                .node()
                .children_by_field_name("declarator", &mut cursor2);

            for decl_node in decl_node_list {
                let name = decl_node
                    .child_by_field_name("name").unwrap()
                    .utf8_text(code).unwrap();

                println!("{}{}: {}", " ".repeat(indent), name, type_name);
            }
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