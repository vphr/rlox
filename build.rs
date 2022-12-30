use std::io::Result;
mod ast_generator;
use ast_generator::*;

fn main() -> Result<()> {
    ast_generator("src")
}
