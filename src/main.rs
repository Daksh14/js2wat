use std::path::PathBuf;

use clap::Parser;

use crate::{lexer::lex, parser::parse};

mod analysis;
mod codegen;
mod lexer;
mod parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// path of the js file to compile to WAT format
    #[arg(short, long)]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    std::fs::write("output.wat", js2wat(load_file(args.path))).unwrap();
}

fn load_file(location: PathBuf) -> String {
    std::fs::read_to_string(location).unwrap()
}

fn js2wat(code: String) -> String {
    let lexed = lex(code.to_owned());

    let parsed = parse(lexed);

    codegen::wat_gen(parsed)
}
