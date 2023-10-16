use tree_sitter::{TreeCursor};

pub struct MetricsMethod {
    name: String,
    cyclomatic: usize,
}

impl MetricsMethod {
    fn new(name: String) -> Self {
        Self {
            name: name,
            cyclomatic: 1,
        }
    }

    fn compute_cyclomatic(&mut self, cursor: &mut TreeCursor, code: &[u8]) {
        match cursor.node().kind() {
            "if_statement" | "while_statement" | "do_statement" | "for_statement" | "enhanced_for_statement" => {
                self.cyclomatic += 1;
            },
            "switch_label" if cursor.node().utf8_text(code).unwrap() != "default" => { // switch statement or expression
                self.cyclomatic += 1;
            },
            "catch_clause" => {
                self.cyclomatic += 1;
            },
            "assert_statement" => { // if (condition) { throw ... }
                self.cyclomatic += 2;
            },
            "throw_statement" => {
                self.cyclomatic += 1;
            },
            "ternary_expression" => {
                self.cyclomatic += 1;
            },
            _ => {},
        }
        
        if cursor.goto_first_child() {
            self.compute_cyclomatic(cursor, code);
            while cursor.goto_next_sibling() {
                self.compute_cyclomatic(cursor, code);
            }
            cursor.goto_parent();
        }
    }

    pub fn dump_metrics(&self) {
        println!("method: {}", self.name);
        println!("cyclo:  {}", self.cyclomatic);
    }
}

pub fn metrics(cursor: &mut TreeCursor, code: &[u8]) -> Vec<MetricsMethod> {
    if cursor.node().kind() == "method_declaration" {
        let mut met = MetricsMethod::new(method_name(cursor, code));
        met.compute_cyclomatic(cursor, code);
        return vec![met];
    }
    
    let mut metrics_list: Vec<MetricsMethod> = Vec::new();
    if cursor.goto_first_child() {
        metrics_list.extend(metrics(cursor, code));
        while cursor.goto_next_sibling() {
            metrics_list.extend(metrics(cursor, code));
        }
        cursor.goto_parent();
    }
    metrics_list
}

fn method_name(cursor: &mut TreeCursor, code: &[u8]) -> String {
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.goto_next_sibling();
    let name = cursor.node().utf8_text(code).unwrap().to_string();
    cursor.goto_parent();

    name
}