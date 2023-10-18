use tree_sitter::{TreeCursor};

pub struct MetricsClass {
    name: String,
    is_class: bool,
    wmc: usize,
    field_name_list: Vec<String>,
    metrics_method_list: Vec<MetricsMethod>,
}

impl MetricsClass {
    fn new(is_class: bool) -> Self {
        Self {
            name: "".to_string(),
            is_class: is_class,
            wmc: 0,
            field_name_list: Vec::new(),
            metrics_method_list: Vec::new(),
        }
    }

    fn compute(&mut self, cursor: &mut TreeCursor, code: &[u8]) {
        self.name = cursor
            .node()
            .child_by_field_name("name").unwrap()
            .utf8_text(code).unwrap().to_string();

        let mut body_cursor = cursor
            .node()
            .child_by_field_name("body").unwrap()
            .walk();

        if self.is_class {
            self.walk_body(&mut body_cursor, code);
        } else {
            body_cursor.goto_first_child();
            loop {
                if body_cursor.node().kind() == "enum_body_declarations" {
                    self.walk_body(&mut body_cursor, code);
                    break;
                }
                if !body_cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        self.compute_wmc();
    }

    fn walk_body(&mut self, cursor: &mut TreeCursor, code: &[u8]) {
        cursor.goto_first_child();
        loop {
            match cursor.node().kind() {
                "field_declaration" => {
                    let mut cursor2 = cursor.clone();
                    let decl_node_list = cursor
                        .node()
                        .children_by_field_name("declarator", &mut cursor2);

                    for decl_node in decl_node_list {
                        let ident = decl_node
                            .child_by_field_name("name").unwrap()
                            .utf8_text(code).unwrap().to_string();

                        self.field_name_list.push(ident);
                    }
                },
                "constructor_declaration" | "method_declaration" => {
                    let mut met = MetricsMethod::new();
                    met.compute(cursor, code);
                    self.metrics_method_list.push(met);
                },
                _ => {},
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }

    fn compute_wmc(&mut self) {
        for metrics_method in &self.metrics_method_list {
            self.wmc += metrics_method.cyclomatic;
        }
    }

    pub fn dump_metrics(&self) {
        println!("");
        println!("{} {}", if self.is_class { "class" } else { "enum" }, self.name);
        println!("    WMC : {}", self.wmc);

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
            .child_by_field_name("name").unwrap()
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
            "lambda_expression" => {
                return;
            },
            "ternary_expression" => {
                self.cyclomatic += 1;
            },
            _ => {},
        }
        
        if cursor.goto_first_child() {
            loop {
                self.compute_cyclomatic(cursor, code);

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    fn dump_metrics(&self) {
        println!("");
        println!("    {}()", self.name);
        println!("        CYCLO: {}", self.cyclomatic);
    }
}

pub fn metrics(cursor: &mut TreeCursor, code: &[u8]) -> Vec<MetricsClass> {
    let mut metrics_list: Vec<MetricsClass> = Vec::new();

    if cursor.node().kind() == "class_declaration" || cursor.node().kind() == "enum_declaration" {
        let mut met = MetricsClass::new(cursor.node().kind() == "class_declaration");
        met.compute(cursor, code);
        metrics_list.push(met);
    }
    
    if cursor.goto_first_child() {
        loop {
            metrics_list.extend(metrics(cursor, code));

            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
    metrics_list
}