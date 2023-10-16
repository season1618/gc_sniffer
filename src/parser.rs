use tree_sitter::{Parser, Language, Tree, TreeCursor};

extern "C" { fn tree_sitter_java() -> Language; }

pub fn parse(code: &String) -> Result<Tree, &str> {
    let mut parser = Parser::new();
    parser.set_language(unsafe { tree_sitter_java() }).unwrap();
    parser.parse(&code.clone(), None).ok_or("failed to parse")
}

pub fn dump_tree(tree_cursor: &mut TreeCursor, indent: usize) {
    println!("{}{}", " ".repeat(indent), tree_cursor.node().kind());

    if tree_cursor.goto_first_child() {
        dump_tree(tree_cursor, indent + 4);
        while tree_cursor.goto_next_sibling() {
            dump_tree(tree_cursor, indent + 4);
        }
        tree_cursor.goto_parent();
    }
}

pub fn dump_attr(tree_cursor: &mut TreeCursor, code: &[u8], indent: usize) {
    let kind = tree_cursor.node().kind();
    if kind == "class_declaration" || kind == "field_declaration" {
        tree_cursor.goto_first_child();
        let type_name = tree_cursor.node().utf8_text(code).unwrap();

        tree_cursor.goto_next_sibling();
        let mut ident = tree_cursor.node().utf8_text(code).unwrap();

        if tree_cursor.goto_first_child() {
            ident = tree_cursor.node().utf8_text(code).unwrap();
            tree_cursor.goto_parent();
        }
        println!("{}{}: {}", " ".repeat(indent), ident, type_name);
        tree_cursor.goto_parent();
    }

    if kind == "method_declaration" {
        tree_cursor.goto_first_child();
        let _modifier = tree_cursor.node().utf8_text(code).unwrap();

        tree_cursor.goto_next_sibling();
        let type_name = tree_cursor.node().utf8_text(code).unwrap();

        tree_cursor.goto_next_sibling();
        let ident = tree_cursor.node().utf8_text(code).unwrap();

        println!("{}{}(): {}", " ".repeat(indent), ident, type_name);
        tree_cursor.goto_parent();
    }

    if tree_cursor.goto_first_child() {
        dump_attr(tree_cursor, code, indent + 4);
        while tree_cursor.goto_next_sibling() {
            dump_attr(tree_cursor, code, indent + 4);
        }
        tree_cursor.goto_parent();
    }
}