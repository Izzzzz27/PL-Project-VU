mod statement;
mod token;
mod tokenizer;
mod parser;
mod error;

use std::io::{self, Write};
use tokenizer::Tokenizer;
use parser::Parser;
use error::Error;

/// Gets a line of input from the user
/// 
/// # Returns
/// A String containing the user's input with whitespace trimmed
fn get_string_from_user() -> String {
    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Attempts to parse an SQL statement from the input string
/// 
/// # Arguments
/// * `input` - The SQL query string to parse
/// 
/// # Returns
/// Result containing either the parsed Statement or an Error
fn build_statement(input: &str) -> Result<statement::Statement, Error> {
    // Create tokenizer and get tokens
    let mut tokenizer = Tokenizer::new(input);
    let tokens = match tokenizer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => return Err(Error::LexerError(e.to_string())),
    };
    
    // Create parser and parse tokens
    let mut parser = Parser::new(tokens);
    parser.parse()
}

fn main() {
    loop {
        let input = get_string_from_user();
        if input.is_empty() {
            break;
        }

        match build_statement(&input) {
            Ok(stmt) => println!("Successfully parsed:\n{:#?}", stmt),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
