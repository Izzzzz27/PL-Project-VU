use crate::token::{Token, Keyword};
use crate::error::Error;
use std::iter::Peekable;
use std::str::Chars;

pub struct Tokenizer<'a> {
    input: Peekable<Chars<'a>>,
    is_after_select: bool,  // Track if we're after SELECT keyword
    current_position: usize,  // Track current position in input
    tokens: Vec<Token>,     // Store all tokens
    current_token: usize,   // Current token index
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
            is_after_select: false,
            current_position: 0,
            tokens: Vec::new(),
            current_token: 0,
        }
    }

    fn advance(&mut self) {
        self.input.next();
        self.current_position += 1;
    }

    fn tokenize_string(&mut self, quote: char) -> Result<Token, Error> {
        self.advance(); // consume opening quote
        let mut string = String::new();
        
        while let Some(&c) = self.input.peek() {
            if c == quote {
                self.advance(); // consume closing quote
                return Ok(Token::String(string));
            }
            string.push(c);
            self.advance();
        }
        
        Err(Error::LexerError(format!("Unterminated string literal starting with {}", quote)))
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();
        
        while let Some(&c) = self.input.peek() {
            let token = match c {
                ' ' | '\t' | '\n' | '\r' => {
                    self.advance();
                    continue;
                }
                '(' => {
                    self.advance();
                    Token::LeftParentheses
                }
                ')' => {
                    self.advance();
                    Token::RightParentheses
                }
                '>' => {
                    self.advance();
                    if let Some(&'=') = self.input.peek() {
                        self.advance();
                        Token::GreaterThanOrEqual
                    } else {
                        Token::GreaterThan
                    }
                }
                '<' => {
                    self.advance();
                    if let Some(&'=') = self.input.peek() {
                        self.advance();
                        Token::LessThanOrEqual
                    } else {
                        Token::LessThan
                    }
                }
                '=' => {
                    self.advance();
                    Token::Equal
                }
                '!' => {
                    self.advance();
                    if let Some(&'=') = self.input.peek() {
                        self.advance();
                        Token::NotEqual
                    } else {
                        return Err(Error::LexerError("Expected '=' after '!'".to_string()));
                    }
                }
                '*' => {
                    self.advance();
                    if self.is_after_select && tokens.last().map_or(true, |t| 
                        matches!(t, Token::Keyword(Keyword::Select)) || matches!(t, Token::Comma)
                    ) {
                        Token::Wildcard
                    } else {
                        Token::Star
                    }
                }
                '/' => {
                    self.advance();
                    Token::Divide
                }
                '-' => {
                    self.advance();
                    Token::Minus
                }
                '+' => {
                    self.advance();
                    Token::Plus
                }
                ',' => {
                    self.advance();
                    Token::Comma
                }
                ';' => {
                    self.advance();
                    Token::Semicolon
                }
                '\'' | '"' => self.tokenize_string(c)?,
                '0'..='9' => {
                    let mut number = 0u64;
                    while let Some(&c) = self.input.peek() {
                        if !c.is_ascii_digit() {
                            break;
                        }
                        if let Some(new_number) = number.checked_mul(10).and_then(|n| n.checked_add(c.to_digit(10).unwrap() as u64)) {
                            number = new_number;
                        } else {
                            return Err(Error::LexerError("Number too large".to_string()));
                        }
                        self.advance();
                    }
                    Token::Number(number)
                }
                'A'..='Z' | 'a'..='z' | '_' => {
                    let mut identifier = String::new();
                    while let Some(&c) = self.input.peek() {
                        if !c.is_ascii_alphanumeric() && c != '_' {
                            break;
                        }
                        identifier.push(c.to_ascii_uppercase());
                        self.advance();
                    }
                    
                    match identifier.as_str() {
                        "SELECT" => {
                            self.is_after_select = true;
                            Token::Keyword(Keyword::Select)
                        }
                        "CREATE" => Token::Keyword(Keyword::Create),
                        "TABLE" => Token::Keyword(Keyword::Table),
                        "WHERE" => Token::Keyword(Keyword::Where),
                        "ORDER" => Token::Keyword(Keyword::Order),
                        "BY" => Token::Keyword(Keyword::By),
                        "ASC" => Token::Keyword(Keyword::Asc),
                        "DESC" => Token::Keyword(Keyword::Desc),
                        "FROM" => {
                            self.is_after_select = false;
                            Token::Keyword(Keyword::From)
                        }
                        "AND" => Token::Keyword(Keyword::And),
                        "OR" => Token::Keyword(Keyword::Or),
                        "NOT" => Token::Keyword(Keyword::Not),
                        "TRUE" => Token::Keyword(Keyword::True),
                        "FALSE" => Token::Keyword(Keyword::False),
                        "PRIMARY" => Token::Keyword(Keyword::Primary),
                        "KEY" => Token::Keyword(Keyword::Key),
                        "FOREIGN" => Token::Keyword(Keyword::Foreign),
                        "REFERENCES" => Token::Keyword(Keyword::References),
                        "CHECK" => Token::Keyword(Keyword::Check),
                        "INT" => Token::Keyword(Keyword::Int),
                        "BOOL" => Token::Keyword(Keyword::Bool),
                        "VARCHAR" => Token::Keyword(Keyword::Varchar),
                        "NULL" => Token::Keyword(Keyword::Null),
                        "INDEX" => Token::Keyword(Keyword::Index),
                        "UNIQUE" => Token::Keyword(Keyword::Unique),
                        "ON" => Token::Keyword(Keyword::On),
                        _ => Token::Identifier(identifier),
                    }
                }
                c => return Err(Error::LexerError(format!("Invalid character: {}", c))),
            };
            tokens.push(token);
        }
        
        tokens.push(Token::Eof);
        Ok(tokens)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tokens.is_empty() {
            match self.tokenize() {
                Ok(tokens) => {
                    self.tokens = tokens;
                    self.current_token = 0;
                }
                Err(e) => return Some(Err(e)),
            }
        }

        if self.current_token >= self.tokens.len() {
            None
        } else {
            let token = self.tokens[self.current_token].clone();
            self.current_token += 1;
            Some(Ok(token))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_char_tokens() {
        let mut tokenizer = Tokenizer::new("+ - * / = < > ( ) , ;");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Divide);
        assert_eq!(tokens[4], Token::Equal);
        assert_eq!(tokens[5], Token::LessThan);
        assert_eq!(tokens[6], Token::GreaterThan);
        assert_eq!(tokens[7], Token::LeftParentheses);
        assert_eq!(tokens[8], Token::RightParentheses);
        assert_eq!(tokens[9], Token::Comma);
        assert_eq!(tokens[10], Token::Semicolon);
    }

    #[test]
    fn test_multi_char_tokens() {
        let mut tokenizer = Tokenizer::new(">= <= != ==");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::GreaterThanOrEqual);
        assert_eq!(tokens[1], Token::LessThanOrEqual);
        assert_eq!(tokens[2], Token::NotEqual);
        assert_eq!(tokens[3], Token::Equal);
    }

    #[test]
    fn test_numbers() {
        let mut tokenizer = Tokenizer::new("42 123 0 9999");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number(42));
        assert_eq!(tokens[1], Token::Number(123));
        assert_eq!(tokens[2], Token::Number(0));
        assert_eq!(tokens[3], Token::Number(9999));
    }

    #[test]
    fn test_strings() {
        let mut tokenizer = Tokenizer::new("'hello' \"world\"");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        assert_eq!(tokens[1], Token::String("world".to_string()));
    }

    #[test]
    fn test_keywords() {
        let mut tokenizer = Tokenizer::new("SELECT FROM WHERE ORDER BY CREATE TABLE INT VARCHAR BOOL");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Keyword(Keyword::Select));
        assert_eq!(tokens[1], Token::Keyword(Keyword::From));
        assert_eq!(tokens[2], Token::Keyword(Keyword::Where));
        assert_eq!(tokens[3], Token::Keyword(Keyword::Order));
        assert_eq!(tokens[4], Token::Keyword(Keyword::By));
        assert_eq!(tokens[5], Token::Keyword(Keyword::Create));
        assert_eq!(tokens[6], Token::Keyword(Keyword::Table));
        assert_eq!(tokens[7], Token::Keyword(Keyword::Int));
        assert_eq!(tokens[8], Token::Keyword(Keyword::Varchar));
        assert_eq!(tokens[9], Token::Keyword(Keyword::Bool));
    }

    #[test]
    fn test_identifiers() {
        let mut tokenizer = Tokenizer::new("username age_2 first_name _temp");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Identifier("USERNAME".to_string()));
        assert_eq!(tokens[1], Token::Identifier("AGE_2".to_string()));
        assert_eq!(tokens[2], Token::Identifier("FIRST_NAME".to_string()));
        assert_eq!(tokens[3], Token::Identifier("_TEMP".to_string()));
    }

    #[test]
    fn test_select_star() {
        let mut tokenizer = Tokenizer::new("SELECT * FROM users");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Keyword(Keyword::Select));
        assert_eq!(tokens[1], Token::Wildcard);
        assert_eq!(tokens[2], Token::Keyword(Keyword::From));
        assert_eq!(tokens[3], Token::Identifier("USERS".to_string()));
    }

    #[test]
    fn test_error_unmatched_quotes() {
        let mut tokenizer = Tokenizer::new("SELECT * FROM users WHERE name = 'John");
        assert!(matches!(tokenizer.tokenize(), Err(Error::LexerError(_))));
    }

    #[test]
    fn test_error_invalid_char() {
        let mut tokenizer = Tokenizer::new("SELECT @ FROM users");
        assert!(matches!(tokenizer.tokenize(), Err(Error::LexerError(_))));
    }

    #[test]
    fn test_foreign_key() {
        let mut tokenizer = Tokenizer::new("FOREIGN KEY (user_id) REFERENCES users(id)");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Keyword(Keyword::Foreign));
        assert_eq!(tokens[1], Token::Keyword(Keyword::Key));
        assert_eq!(tokens[2], Token::LeftParentheses);
        assert_eq!(tokens[3], Token::Identifier("USER_ID".to_string()));
        assert_eq!(tokens[4], Token::RightParentheses);
        assert_eq!(tokens[5], Token::Keyword(Keyword::References));
        assert_eq!(tokens[6], Token::Identifier("USERS".to_string()));
    }

    #[test]
    fn test_iterator() {
        let mut tokenizer = Tokenizer::new("SELECT id FROM users");
        let mut tokens = Vec::new();
        while let Some(token) = tokenizer.next() {
            tokens.push(token.unwrap());
        }
        assert_eq!(tokens[0], Token::Keyword(Keyword::Select));
        assert_eq!(tokens[1], Token::Identifier("ID".to_string()));
        assert_eq!(tokens[2], Token::Keyword(Keyword::From));
        assert_eq!(tokens[3], Token::Identifier("USERS".to_string()));
        assert_eq!(tokens[4], Token::Eof);
    }
}
