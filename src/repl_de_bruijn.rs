use crate::parser::Parser;
use crate::syntax::{DeBruijnExpr};
use crate::{eval_de_bruijn::eval_expr, eval_de_bruijn::Env};
use std::io::{self, Write};
fn parse_expr(s: &str) -> Result<DeBruijnExpr, String> {
    let from_str = Parser::from_str(s);

    match from_str {
        Ok(mut parser) => match parser.parse_expr() {
            Ok(coord_expr) =>  Ok(DeBruijnExpr::from_expr(coord_expr.expr)),
            Err(e) => Err(format!("{}", e)),
        },
        Err(e) => Err(format!("{}", e)),
    }
}

pub fn repl() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("读取失败");
            continue;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        match parse_expr(input) {
            Ok(expr) => {
                let result = eval_expr(expr.clone(), &Env::new());
                match result {
                    Ok(v) => println!("=> {}", v),
                    Err(e) => {
                        println!("{}", e);
                    }
                };
            }
            Err(e) => println!("{}", e),
        }
    }
}
