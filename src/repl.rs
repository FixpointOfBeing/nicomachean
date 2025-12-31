use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor };
use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub parser);

pub fn repl() {
    const HISTORY_FILE: &str = ".history.txt";

    let mut rl = DefaultEditor::new().unwrap();
    if rl.load_history(HISTORY_FILE).is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                match parser::ExprParser::new().parse(line.trim()) {
                    Ok(expr) => {
                         println!("{:?}", *expr);
                    },
                    Err(e) => println!("Parse error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(HISTORY_FILE).unwrap();
}