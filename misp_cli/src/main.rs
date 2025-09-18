use misp_executor::Executor;
use misp_lexer::Lexer;
use misp_parser::Parser;
use rustyline::{DefaultEditor, error::ReadlineError};

fn main() {
    let mut executor = Executor::default();
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline("misp >> ");
        match readline {
            Ok(line) => {
                let lexer = Lexer::default();
                let tokens = lexer.lex(&line).unwrap();

                let mut parser = Parser::new(tokens);
                let sexprs = parser.parse().unwrap();

                for sexpr in sexprs {
                    let res = executor.execute(&sexpr).unwrap();
                    println!("{res}");
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
                println!("Error: {err:?}");
                break;
            }
        }
    }
}
