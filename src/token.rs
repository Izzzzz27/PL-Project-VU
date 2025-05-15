use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Keyword(Keyword),
    
    // Identifiers and literals
    Identifier(String),
    String(String),
    Number(u64),
    
    // Operators and punctuation
    Plus,
    Minus,
    Star,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    LeftParentheses,
    RightParentheses,
    Comma,
    Semicolon,
    Wildcard,
    
    // Special tokens
    Eof,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Keyword {
    Select,
    Create,
    Table,
    Where,
    Order,
    By,
    Asc,
    Desc,
    From,
    And,
    Or,
    Not,
    True,
    False,
    Primary,
    Key,
    Foreign,
    References,
    Check,
    Int,
    Bool,
    Varchar,
    Null,
    Index,
    Unique,
    On,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Keyword(keyword) => write!(f, "{}", keyword),
            Token::Identifier(iden) => write!(f, "{:?}", iden),
            Token::String(str) => write!(f, "{:?}", str),
            Token::Number(num) => write!(f, "{:?}", num),
            Token::RightParentheses => write!(f, "("),
            Token::LeftParentheses => write!(f, ")"),
            Token::GreaterThan => write!(f, ">"),
            Token::GreaterThanOrEqual => write!(f, ">="),
            Token::LessThan => write!(f, "<"),
            Token::LessThanOrEqual => write!(f, "<="),
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "!="),
            Token::Star => write!(f, "*"),
            Token::Wildcard => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Minus => write!(f, "-"),
            Token::Plus => write!(f, "+"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Eof => write!(f, "Eof"),
        }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Select => write!(f, "Select"),
            Keyword::Create => write!(f, "Create"),
            Keyword::Table => write!(f, "Table"),
            Keyword::Where => write!(f, "Where"),
            Keyword::Order => write!(f, "Order"),
            Keyword::By => write!(f, "By"),
            Keyword::Asc => write!(f, "Asc"),
            Keyword::Desc => write!(f, "Desc"),
            Keyword::From => write!(f, "From"),
            Keyword::And => write!(f, "And"),
            Keyword::Or => write!(f, "Or"),
            Keyword::Not => write!(f, "Not"),
            Keyword::True => write!(f, "True"),
            Keyword::False => write!(f, "False"),
            Keyword::Primary => write!(f, "Primary"),
            Keyword::Key => write!(f, "Key"),
            Keyword::Foreign => write!(f, "Foreign"),
            Keyword::References => write!(f, "References"),
            Keyword::Check => write!(f, "Check"),
            Keyword::Int => write!(f, "Int"),
            Keyword::Bool => write!(f, "Bool"),
            Keyword::Varchar => write!(f, "Varchar"),
            Keyword::Null => write!(f, "Null"),
            Keyword::Index => write!(f, "Index"),
            Keyword::Unique => write!(f, "Unique"),
            Keyword::On => write!(f, "On"),
        }
    }
}