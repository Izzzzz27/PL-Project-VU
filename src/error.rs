/// Error types for the SQL Parser
/// This module defines all possible errors that can occur during lexing and parsing.
use thiserror::Error;

/// Represents all possible errors in the SQL Parser
#[derive(Error, Debug)]
pub enum Error {
    /// Error during lexical analysis (tokenization)
    #[error("Lexer error: {0}")]
    LexerError(String),
    
    /// Error during parsing
    #[error("Parser error: {0}")]
    ParserError(String),
    
    /// Unexpected token encountered during parsing
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        /// What was expected by the parser
        expected: String,
        /// What was actually found in the input
        found: String,
    },
    
    /// Unexpected end of input
    #[error("Unexpected end of input")]
    UnexpectedEOF,

    #[error("Invalid VARCHAR length: {0}")]
    InvalidVarcharLength(String),

    #[error("Missing FROM clause in SELECT statement")]
    MissingFromClause,

    #[error("Invalid FOREIGN KEY constraint: {0}")]
    InvalidForeignKey(String),
} 