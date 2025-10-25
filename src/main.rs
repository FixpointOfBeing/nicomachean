mod eval;
mod eval_de_bruijn;
mod lexer;
mod parser;
mod repl;
mod repl_de_bruijn;
mod syntax;
mod type_check;
mod type_check_de_bruijn;
use crate::eval::eval_program;
use crate::parser::Parser as IceParser;
use crate::type_check::check_program;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "File")]
    /// Source File
    file: PathBuf, 
}
fn eval_file() {
    let args = Args::parse();
    let program = IceParser::from_file(&args.file)
        .unwrap()
        .parse_program()
        .unwrap()
        .program;
    let check = check_program(&program);

    if check.is_err() {
        for e in check.err().unwrap() {
            eprintln!("{}", e);
        }
        return;
    }
    let res = eval_program(program);
    match res {
        Ok(v) => println!("{} : {}", v, check.ok().unwrap()),
        Err(e) => eprintln!("{}", e),
    };
}
fn debruijn_eval_file() {
    let args = Args::parse();
    let program = IceParser::from_file(&args.file)
        .unwrap()
        .parse_program()
        .unwrap()
        .program;
    let check = type_check_de_bruijn::check_program(&program);
    if check.is_err() {
        for e in check.err().unwrap() {
            eprintln!("{}", e);
        }
        return;
    }
    let res = eval_de_bruijn::eval_program(program);
    match res {
        Ok(v) => println!("{} : {}", v.to_expr().unwrap(), check.ok().unwrap()),
        Err(e) => eprintln!("{}", e),
    };
}
fn main() {
    // repl::repl();
    // repl_de_bruijn::repl();
    // eval_file();
    debruijn_eval_file();
}
