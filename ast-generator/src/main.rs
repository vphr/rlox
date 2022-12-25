use std::{fs::File, io::Write};

#[derive(Debug)]
struct TreeType {
    base_name: String,
    class_name: String,
    fields: Vec<String>,
}

impl TreeType {
    fn new(base_name: String, class_name: String, fields: Vec<String>) -> Self {
        Self {
            base_name,
            class_name,
            fields,
        }
    }
}
fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        std::process::exit(64);
    }
    let output_dir = args.get(1).unwrap();
    define_ast(
        &output_dir,
        "Expr",
        vec![
            "Binary   : Box<Expr> left, Token operator, Box<Expr> right",
            "Grouping : Box<Expr> expression",
            "Literal  : Literal value",
            "Unary    : Token operator, Box<Expr> right",
        ],
    )?;
    Ok(())
}
fn define_ast(output_dir: &str, filename: &str, types_vec: Vec<&str>) -> std::io::Result<()> {
    let path = format!("{}/{}.rs", output_dir, filename.to_lowercase());
    let mut file = File::create(path)?;
    let mut tree_types: Vec<TreeType> = Vec::new();
    write!(file, "{}", "use crate::scanner::*;\n")?;
    for types in types_vec {
        let (base_name, fields) = types
            .split_once(":")
            .map(|(a, b)| (a.trim(), b.trim()))
            .unwrap();
        let fields: Vec<String> = fields
            .split(",")
            .map(|s| {
                let (name, field_type) = s.trim().split_once(" ").unwrap();
                format!("{}: {}", field_type, name)
            })
            .collect();
        let class_name = format!("{}{}", base_name, filename);
        tree_types.push(TreeType::new(base_name.to_string(), class_name, fields))
    }
    write!(file, "#[derive(Debug)]\n")?;
    write!(file, "pub enum {} {{\n", filename)?;
    for t in &tree_types {
        write!(file, "\t{}({}),\n", t.base_name, t.class_name)?;
    }
    write!(file, "}}\n")?;

    for t in &tree_types {
        write!(file, "#[derive(Debug)]\n")?;
        write!(file, "pub struct {} {{\n", t.class_name)?;
        for f in &t.fields {
            write!(file, "\t{},\n", f)?;
        }
        write!(file, "}}\n\n")?;
    }
    write!(file, "pub trait {}Visitor<T> {{\n", filename)?;
    for t in &tree_types {
        let base = t.base_name.to_lowercase();
        write!(
            file,
            "\t fn visit_{}_{}(&self, {}: &{}) -> T;\n",
            base,
            filename.to_lowercase(),
            base,
            t.class_name
        )?;
    }
    write!(file, "}}\n\n")?;

    for t in &tree_types {
        write!(file, "impl {} {{\n", t.class_name)?;
        write!(
            file,
            "\tfn accept<T>(&self, visitor: &dyn ExprVisitor<T>) -> T {{\n",
        )?;
        write!(
            file,
            "\t\tvisitor.visit_{}_{}(self) \n",
            t.base_name.to_lowercase(),
            filename.to_lowercase()
        )?;
        write!(file, "\t}}\n")?;
        write!(file, "}}\n\n")?;
    }
    Ok(())
}
