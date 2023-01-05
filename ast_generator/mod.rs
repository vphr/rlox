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
pub fn ast_generator(output_dir: &str) -> std::io::Result<()> {
    define_ast(
        &output_dir,
        "Expr",
        vec!["scanner", "error"],
        vec![
            "Assign     : Token name, Box<Expr> value",
            "Binary     : Box<Expr> left, Token operator, Box<Expr> right",
            "Grouping   : Box<Expr> expression",
            "Literal    : Option<Literal> value",
            "Logical    : Box<Expr> left, Token operator, Box<Expr> right",
            "Unary      : Token operator, Box<Expr> right",
            "Variable   : Token name",
        ],
    )?;
    define_ast(
        &output_dir,
        "Stmt",
        vec!["scanner", "error","expr"],
        vec![
            "Block      : Vec<Stmt> statements",
            "Expression : Box<Expr> expression",
            "If         : Box<Expr> condition, Box<Stmt> then_branch, Option<Box<Stmt>> else_branch",
            "Print      : Box<Expr> expression",
            "Var        : Token name, Option<Box<Expr>> initializer",
            "While      : Box<Expr> condition, Box<Stmt> body"
        ],
    )?;
    Ok(())
}
fn define_ast(output_dir: &str, filename: &str,imports: Vec<&str>, types_vec: Vec<&str>) -> std::io::Result<()> {
    let path = format!("{}/{}.rs", output_dir, filename.to_lowercase());
    let mut file = File::create(path)?;
    let mut tree_types: Vec<TreeType> = Vec::new();
    for import in imports{
    write!(file, "use crate::{}::*;\n", import)?;
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
        let class_name = format!("{}{}", base_name, filename);
        tree_types.push(TreeType::new(base_name.to_string(), class_name, fields))
    }
    write!(file, "#[derive(Debug, Clone)]\n")?;
    write!(file, "pub enum {} {{\n", filename)?;
    for t in &tree_types {
        write!(file, "\t{}({}),\n", t.base_name, t.class_name)?;
    }
    write!(file, "}}\n\n")?;

    write!(file, "impl {} {{\n", filename)?;
    write!(
        file,
        "\tpub fn accept<T>(&self, {}_visitor: &dyn {}Visitor<T>) -> Result<T,RloxError> {{\n",
        filename.to_lowercase(),
        filename
    )?;
    write!(file, "\t\tmatch self {{\n")?;
    for t in &tree_types {
        write!(
            file,
            "\t\t\t {}::{}(expr) => expr.accept({}_visitor),\n",
            filename,
            t.base_name,
            filename.to_lowercase(),
        )?;
    }
    write!(file, "\t\t}}\n")?;
    write!(file, "\t}}\n")?;
    write!(file, "}}\n")?;

    for t in &tree_types {
        write!(file, "#[derive(Debug, Clone)]\n")?;
        write!(file, "pub struct {} {{\n", t.class_name)?;
        for f in &t.fields {
            write!(file, "\t pub {},\n", f)?;
        }
        write!(file, "}}\n\n")?;
    }
    write!(file, "pub trait {}Visitor<T> {{\n", filename)?;
    for t in &tree_types {
        let base = t.base_name.to_lowercase();
        write!(
            file,
            "\t fn visit_{}_{}(&self, {}: &{}) -> Result<T, RloxError>;\n",
            base,
            filename.to_lowercase(),
            filename.to_lowercase(),
            t.class_name
        )?;
    }
    write!(file, "}}\n\n")?;

    for t in &tree_types {
        write!(file, "impl {} {{\n", t.class_name)?;
        write!(
            file,
            "\tfn accept<T>(&self, visitor: &dyn {}Visitor<T>) -> Result<T,RloxError> {{\n",
            filename
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
