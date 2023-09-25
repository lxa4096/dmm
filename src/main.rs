mod lexer;
mod parser;
mod interpreter;
mod humanoid;

use lexer::{Lexer, Token, LexerError};
use parser::Parser;
use interpreter::Interpreter;
use std::io::Write;
use std::io;
use std::fs;
use std::env;

fn print_tokens(text: String) {
    let mut lexer = Lexer::new(&text);
    loop {
        let token_result = lexer.get_next_token();
        match token_result {
            Ok(token) => {
                println!("{}", token);
                if token == Token::EOF {
                    break;
                }
            },
            Err(e) => {
                println!("{:?}", e);
                break;
            }
        }
    }
}
fn print_ast(text: String) {
    let lexer = Lexer::new(&text);
    let mut parser = Parser::new(lexer);
    let tree = parser.parse().unwrap();
    dbg!(tree);
}

fn interpret_text(text: String) {
    let lexer = Lexer::new(&text);
    let parser = Parser::new(lexer);
    let mut interpreter = Interpreter::new(parser, std::env::var("USE_HUMANOIDS").is_err());

    match interpreter.interpret() {
        Ok(()) => {},
        Err(err) => {
            println!("{:?}", err)
        }
    };
}

fn repl() {
    let mut should_quit = false;
    while !should_quit {
        let mut text = String::new();
        
        print!("dmm> ");
        io::stdout().flush().expect("IO Error");
        match io::stdin().read_line(&mut text) {
            Ok(_) => {
                text = text.replace('\n', "");
                interpret_text(text);
            },
            Err(_) => {
                should_quit = true;
            }
        }    
    }
}


fn main() -> Result<(), LexerError>{
    if env::args().len() > 1 {
        // Compile file.
        let path = env::args().nth(1).unwrap();
        let text = fs::read_to_string(path).unwrap();
        if env::args().len() > 2 {
            match env::args().nth(2).unwrap().as_str() {
                "--lexer" => {
                    print_tokens(text);
                },
                "--ast" => {
                    print_ast(text);
                },
                _ => {interpret_text(text);}
            }
        } else {
            interpret_text(text);
        }
    } else {
        // REPL.
        repl();
    }
   
    

    Ok(())
}
