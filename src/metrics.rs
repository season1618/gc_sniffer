use std::collections::BTreeSet;
use tree_sitter::{Node, TreeCursor};

pub struct Metrics {
    metrics_class_list: Vec<MetricsClass>,
}

impl Metrics {
    fn new() -> Self {
        Self {
            metrics_class_list: Vec::new(),
        }
    }

    fn compute(&mut self, cursor: &mut TreeCursor, code: &[u8]) {
        cursor.goto_first_child();
        loop {
            if cursor.node().kind() == "class_declaration" || cursor.node().kind() == "enum_declaration" {
                let mut metrics = MetricsClass::new(cursor.node().kind() == "class_declaration");
                metrics.compute(&cursor.node(), code);
                self.metrics_class_list.push(metrics);
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }

    fn dump_god_class(&self, path: &str) {
        for class in &self.metrics_class_list {
            class.dump_god_class(path);
        }
    }
}

struct MetricsClass { // class or enum
    name: String,
    is_class: bool,
    atfd: usize,
    wmc: usize,
    tcc: f32,
    is_god: bool,
    field_name_list: Vec<String>,
    metrics_class_list: Vec<MetricsClass>, // class or enum
    metrics_method_list: Vec<MetricsMethod>, // method or constructor
    line: usize,
}

impl MetricsClass {
    fn new(is_class: bool) -> Self {
        Self {
            name: "".to_string(),
            is_class: is_class,
            atfd: 0,
            wmc: 0,
            tcc: 0.0,
            is_god: false,
            field_name_list: Vec::new(),
            metrics_class_list: Vec::new(),
            metrics_method_list: Vec::new(),
            line: 0,
        }
    }

    fn compute(&mut self, node: &Node, code: &[u8]) {
        self.name = node
            .child_by_field_name("name").unwrap()
            .utf8_text(code).unwrap().to_string();
        
        self.line = node.child_by_field_name("name").unwrap().start_position().row + 1;

        let mut body_cursor = node
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

        self.compute_atfd(&mut node.walk(), code);
        self.compute_wmc();
        self.compute_tcc();
        self.compute_is_god(5, 47, 1.0 / 3.0);
    }

    fn walk_body(&mut self, cursor: &mut TreeCursor, code: &[u8]) {
        cursor.goto_first_child();
        loop {
            match cursor.node().kind() {
                "class_declaration" | "enum_declaration" => {
                    let mut metrics = MetricsClass::new(cursor.node().kind() == "class_declaration");
                    metrics.compute(&cursor.node(), code);
                    self.metrics_class_list.push(metrics);
                },
                "constructor_declaration" | "method_declaration" => {
                    let mut metrics = MetricsMethod::new();
                    metrics.compute(&cursor.node(), code);
                    self.metrics_method_list.push(metrics);
                },
                "field_declaration" => {
                    cursor.goto_first_child();
                    loop {
                        if cursor.field_name() == Some("declarator") {
                            let name = cursor
                                .node()
                                .child_by_field_name("name").unwrap()
                                .utf8_text(code).unwrap().to_string();

                            self.field_name_list.push(name);
                        }
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                },
                _ => {},
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }

    fn compute_atfd(&mut self, cursor: &mut TreeCursor, code: &[u8]) {
        match cursor.node().kind() {
            "field_access" => {
                let object_name = cursor
                    .node()
                    .child_by_field_name("object").unwrap()
                    .utf8_text(code).unwrap();

                let field_name = cursor
                    .node()
                    .child_by_field_name("field").unwrap()
                    .utf8_text(code).unwrap();

                let is_static = object_name.as_bytes()[0].is_ascii_uppercase();
                if object_name != "this" && object_name != "super" && !is_static && !self.field_name_list.contains(&field_name.to_string()) {
                    self.atfd += 1;
                }
            },
            "method_invocation" => {
                if let Some(object_node) = cursor.node().child_by_field_name("object") {
                    let object_name = object_node.kind();

                    let method_name = cursor
                        .node()
                        .child_by_field_name("name").unwrap()
                        .utf8_text(code).unwrap();

                    let method_args = cursor
                        .node()
                        .child_by_field_name("arguments").unwrap()
                        .utf8_text(code).unwrap();

                    let is_getter = (method_name.starts_with("get") || method_name.starts_with("is")) && method_args.len() == 2;
                    let is_setter = method_name.starts_with("set") && method_args.len() > 2;
                    if object_name != "this" && (is_getter || is_setter) {
                        self.atfd += 1;
                    }
                }
            },
            _ => {},
        }

        if cursor.goto_first_child() {
            self.compute_atfd(cursor, code);
            while cursor.goto_next_sibling() {
                self.compute_atfd(cursor, code);
            }
            cursor.goto_parent();
        }
    }

    fn compute_wmc(&mut self) {
        for metrics_method in &self.metrics_method_list {
            self.wmc += metrics_method.cyclomatic;
        }
    }

    fn compute_tcc(&mut self) {
        let n = self.metrics_method_list.len();
        let mut num_method = 0;

        for i in 0..n {
            if self.metrics_method_list[i].name == self.name { continue; }
            num_method += 1;

            let usage1: &BTreeSet<&String> = &self.metrics_method_list[i].usage_field_list
                .iter()
                .filter(|x| self.field_name_list.contains(x))
                .collect();

            for j in 0..i {
                if self.metrics_method_list[j].name == self.name { continue; }

                let usage2: &BTreeSet<&String> = &self.metrics_method_list[j].usage_field_list
                    .iter()
                    .filter(|x| self.field_name_list.contains(x))
                    .collect();

                if !usage1.is_disjoint(usage2) {
                    self.tcc += 1.0;
                }
            }
        }

        if num_method > 1 {
            self.tcc /= (num_method * (num_method - 1) / 2) as f32;
        }
    }

    fn compute_is_god(&mut self, atfd_min: usize, wmc_min: usize, tcc_max: f32) {
        self.is_god = self.atfd > atfd_min && self.wmc >= wmc_min && self.tcc < tcc_max;
    }

    fn dump_god_class(&self, path: &str) {
        if self.is_god {
            println!("{}:{}:", path, self.line);
            // println!("");
            // println!("{} {}", if self.is_class { "class" } else { "enum" }, self.name);
            // println!("    ATFD: {}", self.atfd);
            // println!("    WMC : {}", self.wmc);
            // println!("    TCC : {:.3}%", self.tcc * 100.0);
        }

        for class in &self.metrics_class_list {
            class.dump_god_class(path);
        }
    }
}

struct MetricsMethod { // method or constructor
    name: String,
    cyclomatic: usize,
    usage_field_list: BTreeSet<String>,
}

impl MetricsMethod {
    fn new() -> Self {
        Self {
            name: "".to_string(),
            cyclomatic: 1,
            usage_field_list: BTreeSet::new(),
        }
    }

    fn compute(&mut self, node: &Node, code: &[u8]) {
        self.name = node
            .child_by_field_name("name").unwrap()
            .utf8_text(code).unwrap().to_string();
        
        self.compute_cyclomatic(&mut node.walk(), code);
        self.compute_usage_field(&mut node.walk(), code);
    }

    fn compute_cyclomatic(&mut self, cursor: &mut TreeCursor, code: &[u8]) {
        match cursor.node().kind() {
            "if_statement" | "while_statement" | "do_statement" | "for_statement" | "ternary_expression" => {
                self.cyclomatic += 1;
                if let Some(cond_node) = cursor.node().child_by_field_name("condition") {
                    self.compute_condition_complexity(&mut cond_node.walk());
                }
            },
            "enhanced_for_statement" => {
                self.cyclomatic += 1;
            },
            "switch_expression" => { // switch statement or expression
                let mut cond_cursor = cursor
                    .node()
                    .child_by_field_name("condition").unwrap()
                    .walk();
                
                self.compute_condition_complexity(&mut cond_cursor);
            },
            "switch_label" if cursor.node().utf8_text(code).unwrap() != "default" => { // switch statement or expression
                self.cyclomatic += 1;
            },
            "catch_clause" | "throw_statement" => {
                self.cyclomatic += 1;
            },
            "lambda_expression" | "assert_statement" => {
                return;
            },
            "class_body" => {
                return;
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

    fn compute_condition_complexity(&mut self, cursor: &mut TreeCursor) {
        match cursor.node().kind() {
            "&&" | "||" => {
                self.cyclomatic += 1;
            },
            _ => {},
        }

        if cursor.goto_first_child() {
            self.compute_condition_complexity(cursor);
            while cursor.goto_next_sibling() {
                self.compute_condition_complexity(cursor);
            }
            cursor.goto_parent();
        }
    }

    fn compute_usage_field(&mut self, cursor: &mut TreeCursor, code: &[u8]) {
        match cursor.node().kind() {
            "identifier" => {
                let ident = cursor
                    .node()
                    .utf8_text(code).unwrap().to_string();

                self.usage_field_list.insert(ident);
            },
            "field_access" => {
                let field_name = cursor
                    .node()
                    .child_by_field_name("field").unwrap()
                    .utf8_text(code).unwrap().to_string();

                self.usage_field_list.insert(field_name);
            },
            _ => {},
        }

        if cursor.goto_first_child() {
            self.compute_usage_field(cursor, code);
            while cursor.goto_next_sibling() {
                self.compute_usage_field(cursor, code);
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

pub fn dump_god_class(node: &Node, code: &[u8], path: &str) {
    let mut metrics = Metrics::new();
    metrics.compute(&mut node.walk(), code);
    metrics.dump_god_class(path);
}