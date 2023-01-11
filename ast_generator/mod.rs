use std::{fs::File, io::Write};

#[derive(Debug)]
struct TreeType {
    base_name: String,
    fields: Vec<String>,
}

impl TreeType {
    fn new(base_name: String, fields: Vec<String>) -> Self {
        Self {
            base_name,
            fields,
        }
    }
}
pub fn ast_generator(output_dir: &str) -> std::io::Result<()> {
    define_ast(
        &output_dir,
        "Expr",
        vec!["scanner"],
        vec![
            "Binary     : Box<Expr> left, Token operator, Box<Expr> right",
            "Call       : Box<Expr> callee,  Box<Vec<Expr>> arguments",
            "Assign     : usize id, String name, Box<Expr> value",
            "Grouping   : Box<Expr> expression",
            "Logical    : Box<Expr> left, Token operator, Box<Expr> right",
            "Unary      : Token operator, Box<Expr> right",
            "Variable   : usize id, String name",
        ],
        Some(vec![
        "Number(f64)",
        "String(String)",
        "Boolean(bool)",
        "Nil",
        ])

    )?;
    define_ast(
        &output_dir,
        "Stmt",
        vec!["expr", "rc"],
        vec![
            "Block      : Vec<Stmt> statements",
            "Expression : Expr expression",
            "If         : Expr condition, Box<Stmt> then_branch, Option<Box<Stmt>> else_branch",
            "Function   : String name, Rc<Vec<String>> parameters, Rc<Vec<Stmt>> body",
            "Print      : Expr expression",
            "Return     : Option<Expr> value",
            "Var        : String name, Option<Expr> initializer",
            "While      : Expr condition, Box<Stmt> body",
        ],
        None,
    )?;
    Ok(())
}
fn define_ast(
    output_dir: &str,
    filename: &str,
    imports: Vec<&str>,
    types_vec: Vec<&str>,
    literals: Option<Vec<&str>>,
) -> std::io::Result<()> {
    let path = format!("{}/{}.rs", output_dir, filename.to_lowercase());
    let mut file = File::create(path)?;
    let mut tree_types: Vec<TreeType> = Vec::new();
    for import in imports {
        if import.eq("rc") {
            write!(file, "use std::rc::Rc;\n")?;
        }else{
        write!(file, "use crate::{}::*;\n", import)?;
        }
    }
    write!(file, "\n\n")?;
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
        tree_types.push(TreeType::new(base_name.to_string(), fields))
    }
    write!(file, "#[derive(Debug, PartialEq)]\n")?;
    write!(file, "pub enum {} {{\n", filename)?;
    if let Some(literal) = literals{
    for lit in &literal {
        write!(file, "\t{},\n",lit)?;
    }
    }
    for t in &tree_types {
        write!(file, "\t{}{{\n",t.base_name)?;
        for f in &t.fields {
            write!(file, "\t {},\n", f)?;
        }
    write!(file, "\t}},\n\n")?;
    }
    write!(file, "}}\n\n")?;

    Ok(())
}
