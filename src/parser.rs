use crate::statement::{Statement, Expression, TableColumn, DBType, Constraint, BinaryOperator, UnaryOperator};
use crate::token::{Token, Keyword};
use crate::error::Error;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Statement, Error> {
        match self.peek() {
            Some(Token::Keyword(Keyword::Select)) => self.parse_select(),
            Some(Token::Keyword(Keyword::Create)) => {
                self.advance();
                match self.peek() {
                    Some(Token::Keyword(Keyword::Table)) => self.parse_create_table(),
                    Some(Token::Keyword(Keyword::Unique)) | Some(Token::Keyword(Keyword::Index)) => self.parse_create_index(),
                    Some(token) => Err(Error::UnexpectedToken {
                        expected: "TABLE or INDEX".to_string(),
                        found: format!("{:?}", token),
                    }),
                    None => Err(Error::UnexpectedEOF),
                }
            },
            Some(token) => Err(Error::UnexpectedToken {
                expected: "SELECT or CREATE".to_string(),
                found: format!("{:?}", token),
            }),
            None => Err(Error::UnexpectedEOF),
        }
    }

    fn parse_select(&mut self) -> Result<Statement, Error> {
        // Consume SELECT
        self.advance();

        // Parse columns
        let columns = self.parse_expressions_list()?;

        // Expect FROM
        if !matches!(self.peek(), Some(Token::Keyword(Keyword::From))) {
            return Err(Error::MissingFromClause);
        }
        self.advance();

        // Parse table name
        let from = match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            Some(token) => return Err(Error::ParserError(format!("Expected table name, found {:?}", token))),
            None => return Err(Error::UnexpectedEOF),
        };

        // Parse optional WHERE clause
        let r#where = if let Some(Token::Keyword(Keyword::Where)) = self.peek() {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Parse optional ORDER BY clause
        let mut orderby = Vec::new();
        if let Some(Token::Keyword(Keyword::Order)) = self.peek() {
            self.advance();
            self.expect_keyword(Keyword::By)?;
            
            loop {
                let expr = self.parse_expression()?;
                
                // Check for ASC/DESC
                let expr = match self.peek() {
                    Some(Token::Keyword(Keyword::Asc)) => {
                        self.advance();
                        Expression::UnaryOperation {
                            operand: Box::new(expr),
                            operator: UnaryOperator::Asc,
                        }
                    }
                    Some(Token::Keyword(Keyword::Desc)) => {
                        self.advance();
                        Expression::UnaryOperation {
                            operand: Box::new(expr),
                            operator: UnaryOperator::Desc,
                        }
                    }
                    _ => expr,
                };
                
                orderby.push(expr);
                
                if let Some(Token::Comma) = self.peek() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Expect semicolon
        self.expect_token(Token::Semicolon)?;

        Ok(Statement::Select {
            columns,
            from,
            r#where,
            orderby,
        })
    }

    fn parse_create_table(&mut self) -> Result<Statement, Error> {
        // Expect TABLE
        self.expect_keyword(Keyword::Table)?;

        // Parse table name
        let table_name = match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            Some(token) => return Err(Error::ParserError(format!("Expected table name, found {:?}", token))),
            None => return Err(Error::UnexpectedEOF),
        };

        // Expect left parenthesis
        self.expect_token(Token::LeftParentheses)?;

        // Parse column definitions and table constraints
        let mut column_list: Vec<TableColumn> = Vec::new();
        loop {
            if let Some(Token::RightParentheses) = self.peek() {
                self.advance();
                break;
            }

            if !column_list.is_empty() {
                self.expect_token(Token::Comma)?;
            }

            // Check if it's a FOREIGN KEY constraint
            if let Some(Token::Keyword(Keyword::Foreign)) = self.peek() {
                self.advance();
                self.expect_keyword(Keyword::Key)?;
                
                // Parse (column)
                self.expect_token(Token::LeftParentheses)?;
                let column = match self.peek() {
                    Some(Token::Identifier(name)) => {
                        let name = name.clone();
                        self.advance();
                        name
                    }
                    Some(token) => return Err(Error::InvalidForeignKey(format!("Expected column name, found {:?}", token))),
                    None => return Err(Error::InvalidForeignKey("Missing column name".to_string())),
                };
                self.expect_token(Token::RightParentheses)?;

                // Parse REFERENCES table(column)
                self.expect_keyword(Keyword::References)?;
                let referenced_table = match self.peek() {
                    Some(Token::Identifier(name)) => {
                        let name = name.clone();
                        self.advance();
                        name
                    }
                    Some(token) => return Err(Error::InvalidForeignKey(format!("Expected table name, found {:?}", token))),
                    None => return Err(Error::InvalidForeignKey("Missing referenced table name".to_string())),
                };

                self.expect_token(Token::LeftParentheses)?;
                let referenced_column = match self.peek() {
                    Some(Token::Identifier(name)) => {
                        let name = name.clone();
                        self.advance();
                        name
                    }
                    Some(token) => return Err(Error::InvalidForeignKey(format!("Expected column name, found {:?}", token))),
                    None => return Err(Error::InvalidForeignKey("Missing referenced column name".to_string())),
                };
                self.expect_token(Token::RightParentheses)?;

                // Find the column and add the foreign key constraint
                let mut found = false;
                for col in &mut column_list {
                    if col.column_name == column {
                        col.constraints.push(Constraint::ForeignKey {
                            column: column.clone(),
                            referenced_table: referenced_table.clone(),
                            referenced_column: referenced_column.clone(),
                        });
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(Error::InvalidForeignKey(format!("Column {} not found in table", column)));
                }
                continue;
            }

            column_list.push(self.parse_column_definition()?);
        }

        // Expect semicolon
        self.expect_token(Token::Semicolon)?;

        Ok(Statement::CreateTable {
            table_name,
            column_list,
        })
    }

    fn parse_create_index(&mut self) -> Result<Statement, Error> {
        let is_unique = match self.peek() {
            Some(Token::Keyword(Keyword::Unique)) => {
                self.advance();
                true
            }
            _ => false
        };

        self.expect_keyword(Keyword::Index)?;

        // Parse index name
        let index_name = match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            Some(token) => return Err(Error::ParserError(format!("Expected index name, found {:?}", token))),
            None => return Err(Error::UnexpectedEOF),
        };

        // Expect ON
        self.expect_keyword(Keyword::On)?;

        // Parse table name
        let table_name = match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            Some(token) => return Err(Error::ParserError(format!("Expected table name, found {:?}", token))),
            None => return Err(Error::UnexpectedEOF),
        };

        // Parse (column_name)
        self.expect_token(Token::LeftParentheses)?;
        let column_name = match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            Some(token) => return Err(Error::ParserError(format!("Expected column name, found {:?}", token))),
            None => return Err(Error::UnexpectedEOF),
        };
        self.expect_token(Token::RightParentheses)?;
        self.expect_token(Token::Semicolon)?;

        Ok(Statement::CreateIndex {
            is_unique,
            index_name,
            table_name,
            column_name,
        })
    }

    fn parse_column_definition(&mut self) -> Result<TableColumn, Error> {
        // Parse column name
        let column_name = match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            Some(token) => return Err(Error::ParserError(format!("Expected column name, found {:?}", token))),
            None => return Err(Error::UnexpectedEOF),
        };

        // Parse column type
        let column_type = match self.peek() {
            Some(Token::Keyword(Keyword::Int)) => {
                self.advance();
                DBType::Int
            }
            Some(Token::Keyword(Keyword::Bool)) => {
                self.advance();
                DBType::Bool
            }
            Some(Token::Keyword(Keyword::Varchar)) => {
                self.advance();
                match self.peek() {
                    Some(Token::LeftParentheses) => {
                        self.advance();
                        let size = match self.peek() {
                            Some(Token::Number(n)) => {
                                let value = n.clone();
                                self.advance();
                                value as usize
                            }
                            Some(token) => return Err(Error::InvalidVarcharLength(format!("Expected number, found {:?}", token))),
                            None => return Err(Error::InvalidVarcharLength("Missing VARCHAR length".to_string())),
                        };
                        self.expect_token(Token::RightParentheses)?;
                        DBType::Varchar(size)
                    }
                    _ => return Err(Error::InvalidVarcharLength("Missing VARCHAR length specification".to_string())),
                }
            }
            Some(token) => return Err(Error::ParserError(format!("Expected type, found {:?}", token))),
            None => return Err(Error::UnexpectedEOF),
        };

        // Parse constraints
        let mut constraints = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Keyword(Keyword::Primary)) => {
                    self.advance();
                    self.expect_keyword(Keyword::Key)?;
                    constraints.push(Constraint::PrimaryKey);
                }
                Some(Token::Keyword(Keyword::Foreign)) => {
                    self.advance();
                    self.expect_keyword(Keyword::Key)?;
                    
                    // Parse (column)
                    self.expect_token(Token::LeftParentheses)?;
                    let column = match self.peek() {
                        Some(Token::Identifier(name)) => {
                            let name = name.clone();
                            self.advance();
                            name
                        }
                        Some(token) => return Err(Error::InvalidForeignKey(format!("Expected column name, found {:?}", token))),
                        None => return Err(Error::InvalidForeignKey("Missing column name".to_string())),
                    };
                    self.expect_token(Token::RightParentheses)?;

                    // Parse REFERENCES table(column)
                    self.expect_keyword(Keyword::References)?;
                    let referenced_table = match self.peek() {
                        Some(Token::Identifier(name)) => {
                            let name = name.clone();
                            self.advance();
                            name
                        }
                        Some(token) => return Err(Error::InvalidForeignKey(format!("Expected table name, found {:?}", token))),
                        None => return Err(Error::InvalidForeignKey("Missing referenced table name".to_string())),
                    };

                    self.expect_token(Token::LeftParentheses)?;
                    let referenced_column = match self.peek() {
                        Some(Token::Identifier(name)) => {
                            let name = name.clone();
                            self.advance();
                            name
                        }
                        Some(token) => return Err(Error::InvalidForeignKey(format!("Expected column name, found {:?}", token))),
                        None => return Err(Error::InvalidForeignKey("Missing referenced column name".to_string())),
                    };
                    self.expect_token(Token::RightParentheses)?;

                    constraints.push(Constraint::ForeignKey {
                        column,
                        referenced_table,
                        referenced_column,
                    });
                }
                Some(Token::Keyword(Keyword::Not)) => {
                    self.advance();
                    self.expect_keyword(Keyword::Null)?;
                    constraints.push(Constraint::NotNull);
                }
                Some(Token::Keyword(Keyword::Check)) => {
                    self.advance();
                    match self.peek() {
                        Some(Token::LeftParentheses) => {
                            self.advance();
                            let expr = self.parse_expression()?;
                            self.expect_token(Token::RightParentheses)?;
                            constraints.push(Constraint::Check(expr));
                        }
                        _ => return Err(Error::ParserError("Expected '(' after CHECK".to_string())),
                    }
                }
                _ => break,
            }
        }

        Ok(TableColumn {
            column_name,
            column_type,
            constraints,
        })
    }

    fn parse_expressions_list(&mut self) -> Result<Vec<Expression>, Error> {
        let mut expressions = Vec::new();

        // Handle SELECT * case
        if let Some(Token::Wildcard) = self.peek() {
            self.advance();
            expressions.push(Expression::Identifier("*".to_string()));
            
            match self.peek() {
                Some(Token::Keyword(Keyword::From)) => return Ok(expressions),
                Some(token) => return Err(Error::UnexpectedToken {
                    expected: "FROM".to_string(),
                    found: format!("{:?}", token),
                }),
                None => return Err(Error::UnexpectedEOF),
            }
        }

        // Normal expression list parsing
        loop {
            // Parse an expression
            let expr = self.parse_expression()?;
            expressions.push(expr);

            match self.peek() {
                Some(Token::Comma) => {
                    self.advance();
                    continue;
                }
                Some(Token::Keyword(Keyword::From)) => break,
                Some(Token::Semicolon) => break,
                Some(token) => return Err(Error::UnexpectedToken {
                    expected: "comma or FROM".to_string(),
                    found: format!("{:?}", token),
                }),
                None => return Err(Error::UnexpectedEOF),
            }
        }

        Ok(expressions)
    }

    fn parse_expression(&mut self) -> Result<Expression, Error> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, min_precedence: u8) -> Result<Expression, Error> {
        let mut left = self.parse_prefix_expression()?;

        while let Some(token) = self.peek() {
            if token == &Token::Semicolon || token == &Token::Comma || 
               token == &Token::Keyword(Keyword::From) || token == &Token::RightParentheses ||
               token == &Token::Keyword(Keyword::Order) || token == &Token::Keyword(Keyword::Asc) ||
               token == &Token::Keyword(Keyword::Desc) {
                break;
            }
            let precedence = self.get_binary_precedence(token);
            if precedence < min_precedence {
                break;
            }

            let operator = self.parse_binary_operator()?;
            let right = self.parse_binary_expression(precedence + 1)?;

            left = Expression::BinaryOperation {
                left_operand: Box::new(left),
                operator,
                right_operand: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression, Error> {
        match self.peek() {
            Some(Token::Number(n)) => {
                let n = *n;
                self.advance();
                Ok(Expression::Number(n))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::String(s))
            }
            Some(Token::Identifier(i)) => {
                let i = i.clone();
                self.advance();
                Ok(Expression::Identifier(i))
            }
            Some(Token::Keyword(Keyword::True)) => {
                self.advance();
                Ok(Expression::Bool(true))
            }
            Some(Token::Keyword(Keyword::False)) => {
                self.advance();
                Ok(Expression::Bool(false))
            }
            Some(Token::LeftParentheses) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightParentheses)?;
                Ok(expr)
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Minus,
                })
            }
            Some(Token::Plus) => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Plus,
                })
            }
            Some(Token::Keyword(Keyword::Not)) => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(Expression::UnaryOperation {
                    operand: Box::new(expr),
                    operator: UnaryOperator::Not,
                })
            }
            Some(token) => Err(Error::ParserError(format!("Unexpected token in prefix position: {:?}", token))),
            None => Err(Error::UnexpectedEOF),
        }
    }

    fn parse_binary_operator(&mut self) -> Result<BinaryOperator, Error> {
        let op = match self.peek() {
            Some(Token::Plus) => BinaryOperator::Plus,
            Some(Token::Minus) => BinaryOperator::Minus,
            Some(Token::Star) => BinaryOperator::Multiply,
            Some(Token::Divide) => BinaryOperator::Divide,
            Some(Token::Equal) => BinaryOperator::Equal,
            Some(Token::NotEqual) => BinaryOperator::NotEqual,
            Some(Token::GreaterThan) => BinaryOperator::GreaterThan,
            Some(Token::GreaterThanOrEqual) => BinaryOperator::GreaterThanOrEqual,
            Some(Token::LessThan) => BinaryOperator::LessThan,
            Some(Token::LessThanOrEqual) => BinaryOperator::LessThanOrEqual,
            Some(Token::Keyword(Keyword::And)) => BinaryOperator::And,
            Some(Token::Keyword(Keyword::Or)) => BinaryOperator::Or,
            Some(token) => return Err(Error::ParserError(format!("Unexpected token in operator position: {:?}", token))),
            None => return Err(Error::UnexpectedEOF),
        };
        self.advance();
        Ok(op)
    }

    fn get_binary_precedence(&self, token: &Token) -> u8 {
        match token {
            Token::Keyword(Keyword::Or) => 1,
            Token::Keyword(Keyword::And) => 2,
            Token::Equal | Token::NotEqual => 3,
            Token::GreaterThan | Token::GreaterThanOrEqual |
            Token::LessThan | Token::LessThanOrEqual => 4,
            Token::Plus | Token::Minus => 5,
            Token::Star | Token::Divide => 6,
            Token::Semicolon => 0,  // Semicolon has lowest precedence
            _ => 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), Error> {
        match self.peek() {
            Some(token) if token == &expected => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(Error::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: format!("{:?}", token),
            }),
            None => Err(Error::UnexpectedEOF),
        }
    }

    fn expect_keyword(&mut self, expected: Keyword) -> Result<(), Error> {
        match self.peek() {
            Some(Token::Keyword(keyword)) if keyword == &expected => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(Error::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: format!("{:?}", token),
            }),
            None => Err(Error::UnexpectedEOF),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Tokenizer;

    fn parse_sql(input: &str) -> Result<Statement, Error> {
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_select_basic() {
        let stmt = parse_sql("SELECT id, name FROM users;").unwrap();
        match stmt {
            Statement::Select { columns, from, r#where, orderby } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(from, "USERS");
                assert!(r#where.is_none());
                assert!(orderby.is_empty());
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_parse_select_where() {
        let stmt = parse_sql("SELECT id FROM users WHERE age >= 18;").unwrap();
        match stmt {
            Statement::Select { columns: _, from: _, r#where, orderby: _ } => {
                assert!(r#where.is_some());
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_parse_select_order_by() {
        let stmt = parse_sql("SELECT id FROM users ORDER BY name ASC, age DESC;").unwrap();
        match stmt {
            Statement::Select { columns: _, from: _, r#where: _, orderby } => {
                assert_eq!(orderby.len(), 2);
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_parse_select_star() {
        let stmt = parse_sql("SELECT * FROM users;").unwrap();
        match stmt {
            Statement::Select { columns, from: _, r#where: _, orderby: _ } => {
                assert_eq!(columns.len(), 1);
                assert!(matches!(&columns[0], Expression::Identifier(s) if s == "*"));
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_parse_create_table_basic() {
        let stmt = parse_sql("CREATE TABLE users (id INT, name VARCHAR(255));").unwrap();
        match stmt {
            Statement::CreateTable { table_name, column_list } => {
                assert_eq!(table_name, "USERS");
                assert_eq!(column_list.len(), 2);
                assert!(matches!(column_list[0].column_type, DBType::Int));
                assert!(matches!(column_list[1].column_type, DBType::Varchar(255)));
            }
            _ => panic!("Expected CreateTable statement"),
        }
    }

    #[test]
    fn test_parse_create_table_constraints() {
        let stmt = parse_sql("CREATE TABLE users (
            id INT PRIMARY KEY,
            email VARCHAR(255) NOT NULL,
            age INT CHECK(age >= 18)
        );").unwrap();
        match stmt {
            Statement::CreateTable { table_name: _, column_list } => {
                assert_eq!(column_list.len(), 3);
                assert!(column_list[0].constraints.contains(&Constraint::PrimaryKey));
                assert!(column_list[1].constraints.contains(&Constraint::NotNull));
                assert!(matches!(&column_list[2].constraints[0], Constraint::Check(_)));
            }
            _ => panic!("Expected CreateTable statement"),
        }
    }

    #[test]
    fn test_parse_create_table_foreign_key() {
        let stmt = parse_sql("CREATE TABLE orders (
            id INT PRIMARY KEY,
            user_id INT,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );").unwrap();
        match stmt {
            Statement::CreateTable { table_name: _, column_list } => {
                assert!(matches!(&column_list[1].constraints[0], 
                    Constraint::ForeignKey { column, referenced_table, referenced_column }
                    if column == "USER_ID" && referenced_table == "USERS" && referenced_column == "ID"
                ));
            }
            _ => panic!("Expected CreateTable statement"),
        }
    }

    #[test]
    fn test_parse_expressions() {
        let stmt = parse_sql("SELECT id * 2 + 3, (age - 18) / 2 FROM users;").unwrap();
        match stmt {
            Statement::Select { columns, from: _, r#where: _, orderby: _ } => {
                assert_eq!(columns.len(), 2);
                assert!(matches!(&columns[0], Expression::BinaryOperation { .. }));
                assert!(matches!(&columns[1], Expression::BinaryOperation { .. }));
            }
            _ => panic!("Expected Select statement"),
        }
    }

    #[test]
    fn test_error_no_from() {
        assert!(matches!(
            parse_sql("SELECT id;"),
            Err(Error::MissingFromClause)
        ));
    }

    #[test]
    fn test_error_invalid_varchar() {
        assert!(matches!(
            parse_sql("CREATE TABLE users (name VARCHAR);"),
            Err(Error::InvalidVarcharLength(_))
        ));
    }

    #[test]
    fn test_error_invalid_check_constraint() {
        assert!(matches!(
            parse_sql("CREATE TABLE users (age INT CHECK);"),
            Err(Error::ParserError(_))
        ));
    }

    #[test]
    fn test_parse_complex_select() {
        let stmt = parse_sql("SELECT id * 2 + 1, name FROM users WHERE age >= 18 AND (salary > 50000 OR department = 'IT') ORDER BY name DESC;").unwrap();
        match stmt {
            Statement::Select { columns, from, r#where, orderby } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(from, "USERS");
                assert!(r#where.is_some());
                assert_eq!(orderby.len(), 1);
                assert!(matches!(&orderby[0], Expression::UnaryOperation { .. }));
            }
            _ => panic!("Expected Select statement"),
        }
    }
}
