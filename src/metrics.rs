use std::collections::BTreeSet;
use tree_sitter::{TreeCursor};

pub struct MetricsClass {
    name: String,
    is_class: bool,
    atfd: usize,
    wmc: usize,
    tcc: f32,
    field_name_list: Vec<String>,
    method_name_list: Vec<String>,
    metrics_method_list: Vec<MetricsMethod>,
}

impl MetricsClass {
    fn new(is_class: bool) -> Self {
        Self {
            name: "".to_string(),
            is_class: is_class,
            atfd: 0,
            wmc: 0,
            tcc: 0.0,
            field_name_list: Vec::new(),
            method_name_list: Vec::new(),
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

        self.compute_atfd(cursor, code);
        self.compute_wmc();
        self.compute_tcc();
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
                        let name = decl_node
                            .child_by_field_name("name").unwrap()
                            .utf8_text(code).unwrap().to_string();

                        self.field_name_list.push(name);
                    }
                },
                "constructor_declaration" | "method_declaration" => {
                    let mut met = MetricsMethod::new();
                    met.compute(cursor, code);
                    self.metrics_method_list.push(met);

                    let name = cursor
                        .node()
                        .child_by_field_name("name").unwrap()
                        .utf8_text(code).unwrap().to_string();

                    self.method_name_list.push(name);
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
                    .kind();

                let field_name = cursor
                    .node()
                    .child_by_field_name("field").unwrap()
                    .utf8_text(code).unwrap();

                if object_name != "this" && object_name != "super" && field_name != &field_name.to_uppercase() && !self.field_name_list.contains(&field_name.to_string()) {
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
        if n <= 1 {
            return;
        }

        for i in 0..n {
            let usage1: &BTreeSet<&String> = &self.metrics_method_list[i].usage_field_list
                .iter()
                .filter(|x| self.field_name_list.contains(x))
                .collect();

            for j in 0..i {
                let usage2: &BTreeSet<&String> = &self.metrics_method_list[j].usage_field_list
                    .iter()
                    .filter(|x| self.field_name_list.contains(x))
                    .collect();

                if !usage1.is_disjoint(usage2) {
                    self.tcc += 1.0;
                }
            }
        }

        self.tcc /= (n * (n - 1) / 2) as f32;
    }

    pub fn dump_metrics(&self) {
        println!("");
        println!("{} {}", if self.is_class { "class" } else { "enum" }, self.name);
        println!("    ATFD: {}", self.atfd);
        println!("    WMC : {}", self.wmc);
        println!("    TCC : {:.3}%", self.tcc * 100.0);

        // for metrics_method in &self.metrics_method_list {
        //     metrics_method.dump_metrics();
        // }
    }
}

struct MetricsMethod {
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

    fn compute(&mut self, cursor: &mut TreeCursor, code: &[u8]) {
        self.name = cursor
            .node()
            .child_by_field_name("name").unwrap()
            .utf8_text(code).unwrap().to_string();
        
        self.compute_cyclomatic(cursor, code);
        self.compute_usage_field(cursor, code);
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