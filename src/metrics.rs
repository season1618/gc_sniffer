use tree_sitter::{TreeCursor};

pub struct MetricsClass {
    name: String,
    wmc: usize,
    metrics_method_list: Vec<MetricsMethod>,
}

impl MetricsClass {
    fn new() -> Self {
        Self {
            name: "".to_string(),
            wmc: 0,
            metrics_method_list: Vec::new(),
        }
    }

    fn compute(&mut self, cursor: &TreeCursor, code: &[u8]) {
        self.name = cursor
            .node()
            .child_by_field_name("name".as_bytes()).unwrap()
            .utf8_text(code).unwrap().to_string();

        let mut class_cursor = cursor
            .node()
            .child_by_field_name("body".as_bytes()).unwrap()
            .walk();

        class_cursor.goto_first_child();
        if class_cursor.node().kind() == "method_declaration" {
            let mut met = MetricsMethod::new();
            met.compute(&mut class_cursor, code);
            self.metrics_method_list.push(met);
        }
        while class_cursor.goto_next_sibling() {
            if class_cursor.node().kind() == "method_declaration" {
                let mut met = MetricsMethod::new();
                met.compute(&mut class_cursor, code);
                self.metrics_method_list.push(met);
            }
        }

        class_cursor.goto_parent();

        self.compute_wmc();
    }

    fn compute_wmc(&mut self) {
        for metrics_method in &self.metrics_method_list {
            self.wmc += metrics_method.cyclomatic;
        }
    }

    pub fn dump_metrics(&self) {
        println!("class: {}", self.name);
        println!("wmc  : {}", self.wmc);
        println!("");

        for metrics_method in &self.metrics_method_list {
            metrics_method.dump_metrics();
        }
    }
}

struct MetricsMethod {
    name: String,
    cyclomatic: usize,
}

impl MetricsMethod {
    fn new() -> Self {
        Self {
            name: "".to_string(),
            cyclomatic: 1,
        }
    }

    fn compute(&mut self, cursor: &mut TreeCursor, code: &[u8]) {
        self.name = cursor
            .node()
            .child(2).unwrap()
            .utf8_text(code).unwrap().to_string();
        
        self.compute_cyclomatic(cursor, code);
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

    fn dump_metrics(&self) {
        println!("method: {}", self.name);
        println!("cyclo:  {}", self.cyclomatic);
    }
}

pub fn metrics(cursor: &mut TreeCursor, code: &[u8]) -> Vec<MetricsClass> {
    if cursor.node().kind() == "class_declaration" {
        let mut met = MetricsClass::new();
        met.compute(cursor, code);
        return vec![met];
    }
    
    let mut metrics_list: Vec<MetricsClass> = Vec::new();
    if cursor.goto_first_child() {
        metrics_list.extend(metrics(cursor, code));
        while cursor.goto_next_sibling() {
            metrics_list.extend(metrics(cursor, code));
        }
        cursor.goto_parent();
    }
    metrics_list
}