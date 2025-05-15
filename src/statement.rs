use std::fmt::{Debug, Display, Formatter};

/// The main entity of the whole parser. `Statement` is implemented as an enumeration because adding functionality is as easy as adding an enumeration constant and implementing functionality for that enumeration constant (implementation in the database command interpreter, which is not a part of this project). Parsing any correct `SELECT` or `CREATE`  (or `UPDATE`, `INSERT INTO`, ... hypothetically) statement should be turned into an instance of this enumeration. Ultimately, your main parser function (something like `build_statement(query: &str) -> Statement`) should return this enumeration.
///
/// The `SELECT` statement has four components:
/// 1. `columns` – A vector of columns from the selected table that the database should return.
/// 2. `from` – A simple string, containing a table that is being queried (we aren't doing joins because they complicate stuff too much for this project).
/// 3. `where` – A single expression that is the actual filter for the database query. It is wrapped in an `Option` because not every `SELECT` query contains a filter. The actual name is `r#where` because in Rust, `where` is a reserved keyword, and the prefix `r#` means: interpret this token as a raw string, do not check for keyword matches.
/// 4. `orderby` – A vector of expressions that define how should the data be ordered. A vector is needed because the data can be ordered by the first column, and then all data that has the same first column can be ordered by the second column, ... Also, the data can be ordered not simply by columns, but by complex expressions as well.
///
/// The `CREATE TABLE` statement has two components:
/// 1. `table_name` – A simple string, the name of the table.
/// 2. `column_list` – A vector of table column types, where each table column contains the definition of one column.
///
/// Examples:
///
/// ---
/// ```sql
/// SELECT name, surname FROM users;
/// ```
/// is a `SELECT` statement that,  when parsed, looks like this:
/// ```rust
/// Statement::Select {
///     columns: vec![
/// 		Expression::Identifier("name".to_string()),
/// 		Expression:Identifier("surname".to_string())
/// 	],
///     from: "users".to_string(),
///     r#where: None,
///     orderby: vec![]
/// }
/// ```
/// ---
/// ```sql
/// SELECT age * 5, 'this is a string' FROM users;
/// ```
/// is a `SELECT` statement that,  when parsed, looks like this:
/// ```rust
/// Statement::Select {
///     columns: vec![
///         Expression::BinaryOperation {
///             left_operand: Box::new(Expression::Identifier("age".to_string())),
///             operator: BinaryOperator::Multiply,
///             right_operand: Box::new(Expression::Number(5)),
///         },
///         Expression::String("this is a string".to_string()),
///     ],
///     from: "users".to_string(),
///     r#where: None,
///     orderby: vec![]
/// }
/// ```
/// ---
/// ```sql
/// SELECT name, surname FROM users WHERE name = \"Voldemort\" AND surname = 'Riddle';
/// ```
/// is a  `SELECT` statement that, when parsed, looks like this:
/// ```rust
/// Statement::Select {
///     columns: [
///         Expression::Identifier("name".to_string()),
///         Expression::Identifier("surname".to_string()),
///     ],
///     from: "users".to_string(),
///     r#where: Some(
///         Expression::BinaryOperation {
///             left_operand: Box::new(Expression::BinaryOperation {
///                 left_operand: Box::new(Expression::Identifier("name".to_string())),
///                 operand: BinaryOperator::Equals,
///                 right_operand: Box::new(Expression::String("Voldemort".to_string())),
///             }),
///             operand: BinaryOperator::And,
///             right_operand: Box::new(Expression::BinaryOperation {
///                 left_operand: Box::new(Expression::Identifier("surname".to_string())),
///                 operand: BinaryOperator::Equals,
///                 right_operand: Box::new(Expression::String("Riddle".to_string())),
///             }),
///         },
///     ),
///     orderby: vec![]
/// }
/// ```
///  ---
/// ```sql
/// SELECT id, salary FROM users ORDER BY salary - 2 * 10 ASC, id DESC;
/// ```
/// is a  `SELECT` statement that, when parsed, looks like this:
/// ```rust
/// Statement::Select {
///     columns: vec![
///         Expression::Identifier("id".to_string()),
///         Expression::Identifier("salary".to_string()),
///     ],
///     from: "users".to_string(),
///     r#where: None,
///     orderby: vec![
///         Expression::UnaryOperation {
///             operand: Box::new(Expression::BinaryOperation {
///                 left_operand: Box::new(Expression::Identifier("salary".to_string())),
///                 operator: BinaryOperator::Minus,
///                 right_operand: Box::new(Expression::BinaryOperation {
///                     left_operand: Box::new(Expression::Number(2)),
///                     operator: BinaryOperator::Multiply,
///                     right_operand: Box::new(Expression::Number(10)),
///                 }),
///             }),
///             operator: UnaryOperator::Asc,
///         },
///         Expression::UnaryOperation {
///             operand: Box::new(Expression::Identifier("id".to_string())),
///             operator: UnaryOperator::Desc,
///         },
///     ],
/// }
/// ```
///  ---
/// ```sql
/// SELECT id FROM registered_users WHERE password_encryption = TRUE ORDER BY id DESC;
/// ```
/// is a  `SELECT` statement that, when parsed, looks like this:
/// ```rust
/// Statement::Select {
///     columns: vec![
///         Expression::Identifier("id".to_string())
///     ],
///     from: "registered_users".to_string(),
///     r#where: Some(
///         Expression::BinaryOperation {
///             left_operand: Box::new(Expression::Identifier("password_encryption".to_string())),
///             operator: BinaryOperator::Equals,
///             right_operand: Box::new(Expression::Bool(true))
///         }
///     ),
///     orderby: vec![
///         Expression::UnaryOperation {
///             operand: Box::new(Expression::Identifier("id".to_string())),
///             operator: UnaryOperator::Desc
///         }
///     ]
/// }
/// ```
/// ---
/// ```sql
/// CREATE TABLE simple_table(
/// 	int_col INT,
/// 	string_col VARCHAR(255),
/// 	bool_col BOOL
/// );
/// ```
/// is a  `CREATE TABLE` statement that, when parsed, looks like this:
/// ```rust
/// Statement::CreateTable {
///     table_name: "simple_table".to_string(),
///     column_list: vec![
///         TableColumn {
///             column_name: "int_col".to_string(),
///             column_type: DBType::Int,
///             constraints: vec![],
///         },
///         TableColumn {
///             column_name: "string_col".to_string(),
///             column_type: DBType::Varchar(255),
///             constraints: vec![],
///         },
///         TableColumn {
///             column_name: "bool_col".to_string(),
///             column_type: DBType::Bool,
///             constraints: vec![],
///         },
///     ]
/// }
/// ```
/// ---
/// ```sql
/// CREATE TABLE complex_table(
/// 	id INT PRIMARY KEY,
/// 	email VARCHAR(255) NOT NULL,
/// 	is_junior BOOL,
/// 	age INT CHECK(age >= 18) CHECK(age <= 65)
/// );
/// ```
/// is a  `CREATE TABLE` statement that, when parsed, looks like this:
/// ```rust
/// Statement::CreateTable {
///     table_name: Expression::Identifier("complex_table".to_string()),
///     column_list: vec![
///         TableColumn {
///             column_name: Expression::Identifier("id".to_string()),
///             column_type: DBType::Int,
///             constraints: vec![
///                 Constraint::PrimaryKey,
///             ],
///         },
///         TableColumn {
///             column_name: Expression::Identifier("email".to_string()),
///             column_type: DBType::Varchar(255),
///             constraints: vec![
///                 Constraint::NotNull,
///             ],
///         },
///         TableColumn {
///             column_name: Expression::Identifier("is_junior".to_string()),
///             column_type: DBType::Bool,
///             constraints: vec![],
///         },
///         TableColumn {
///             column_name: Expression::Identifier("age".to_string()),
///             column_type: DBType::Int,
///             constraints: vec![
///                 Constraint::Check(Expression::BinaryOperation {
///                     left_operand: Box::new(Expression::Identifier("age".to_string())),
///                     operator: BinaryOperator::GreaterThanOrEqual,
///                     right_operand: Box::new(Expression::Number(18)),
///                 }),
///                 Constraint::Check(Expression::BinaryOperation {
///                     left_operand: Box::new(Expression::Identifier("age".to_string())),
///                     operator: BinaryOperator::LessThanOrEqual,
///                     right_operand: Box::new(Expression::Number(65)),
///                 }),
///             ],
///         },
///     ],
/// }
/// ```
/// ---
/// ```sql
/// SELECT salary WHERE salary > 1000;
/// ```
/// is a string, that, the parser should throw an error to the user when it encounters it (no `FROM` clause).
///
/// ---
/// ```sql
/// CREATE TABLE work_hours(num_hours INT)
/// ```
/// is a string, that, the parser should throw an error to the user when it encounters it (no semicolon at the end).
#[derive(Debug, PartialEq)]
pub enum Statement {
    Select {
        columns: Vec<Expression>,
        from: String,
        r#where: Option<Expression>,
        orderby: Vec<Expression>,
    },
    CreateTable {
        table_name: String,
        column_list: Vec<TableColumn>,
    },
    CreateIndex {
        is_unique: bool,
        index_name: String,
        table_name: String,
        column_name: String,
    }
}

/// The main entity of the expression parser. The Expression enum is structured like this, where an expression can contain another expression. This naturally allows us to represent complex expressions as trees. `Box<T>` smart pointers are used on unary and binary types of expressions because the compiler needs to know the size of the enum at compile time which is impossible when an enum contains itself (infinite size).
///
/// An expression can be:
/// * complex - a number of other expressions (tree-like structure, unary and binary operations)
/// * a single number
/// * a single identifier (like a variable 'x')
/// * a single string (when doing parsing of WHERE statements that do operations with strings, strings must be in matching quotes – either `""` or `''`)
/// * a boolean (only true or false)
///
/// Examples:
///
/// ---
/// ```
/// (13 + 7) - 4
/// ```
/// is an expression that contains two expressions:
/// 1. `(13 + 7)` which is
/// ```rust
/// Expression::BinaryOperation {
///     left_operand: Box::new(Expression::Number(13)),
///     operator: BinaryOperator::Plus,
///     right_operand: Box::new(Expression::Number(7))
/// }
/// ```
/// 2. `4` which is
/// ```rust
/// Expression::Number(4)
/// ```
/// Therefore, the whole expression after parsing should look like this:
/// ```rust
/// Expression::BinaryOperation {
///     left_operand: Expression::BinaryOperation {
///         left_operand: Box::new(Expression::Number(13)),
///         operator: BinaryOperator::Plus,
///         right_operand: Box::new(Expression::Number(7))
///     },
///     operator: BinaryOperator::Minus,
///     right_operand: Box::new(Expression::Number(4))
/// }
/// ```
/// ---
/// ```
/// (5 - x) < (4 + y) OR name = "Donna"
/// ```
/// is an expression that contains five (three small and two combining) expressions:
/// 1. `(5 - x)` which is
/// ```rust
/// Expression::BinaryOperation {
///     left_operand: Box::new(Expression::Number(5)),
///     operator: BinaryOperator::Minus,
///     right_operand: Box::new(Expression::Identifier("x".to_string())),
/// }
/// ```
/// 2. `(4 - y)` which is
/// ```rust
/// Expression::BinaryOperation {
///     left_operand: Box::new(Expression::Number(4)),
///     operator: BinaryOperator::Plus,
///     right_operand: Box::new(Expression::Identifier("y".to_string()))
/// }
/// ```
/// 3. `name = "Donna"` which is
/// ```rust
/// Expression::BinaryOperation {
///     left_operand: Box::new(Expression::Identifier("name".to_string())),
///     operator: BinaryOperator::Equal,
///     right_operand: Box::new(Expression::String("Donna".to_string()))
/// }
/// ```
/// Therefore, the whole expression after parsing should look like this:
/// ```rust
/// Expression::BinaryOperation {
///     left_operand: Box::new(Expression::BinaryOperation {
///         left_operand: Box::new(Expression::BinaryOperation {
///             left_operand: Box::new(Expression::Number(5)),
///             operator: BinaryOperator::Minus,
///             right_operand: Box::new(Expression::Identifier("x".to_string()))
///         }),
///         operator: BinaryOperator::LessThan,
///         right_operand: Box::new(Expression::BinaryOperation {
///             left_operand: Box::new(Expression::Number(4)),
///             operator: BinaryOperator::Plus,
///             right_operand: Box::new(Expression::Identifier("y".to_string()))
///         })
///     }),
///     operator: BinaryOperator::Or,
///     right_operand: Box::new(Expression::BinaryOperation {
///         left_operand: Box::new(Expression::Identifier("name".to_string())),
///         operator: BinaryOperator::Equal,
///         right_operand: Box::new(Expression::String("Donna".to_string()))
///     })
/// }
/// ```
/// ---
/// ```
/// NOT some_boolean = TRUE
/// ```
/// should look like this:
/// ```rust
/// Expression::BinaryOperation {
///     left_operand: Box::new(Expression::UnaryOperation {
///         left_operand: Box::new(Expression::Identifier("some_boolean".to_string())),
///         operator: UnaryOperator::Not
///     }),
///     operator: BinaryOperator::Equal,
///     right_operand: Box::new(Expression::Bool(true))
/// }
/// ```
/// ---
/// ```
/// 5 * 3 - 4 + c / (13 -)
/// ```
/// is a string, that, the parser should throw an error to the user when it encounters it.

#[derive(Debug, PartialEq)]
pub enum Expression {
    BinaryOperation {
        left_operand: Box<Expression>,
        operator: BinaryOperator,
        right_operand: Box<Expression>,
    },
    UnaryOperation {
        operand: Box<Expression>,
        operator: UnaryOperator,
    },
    Number(u64),
    Bool(bool),
    Identifier(String),
    String(String),
}

/// A structure containing a definition for one column, when creating a table.
/// 1. `column_name` – A simple string, representing a name.
/// 2. `column_type` – The type of the column. Types are defined in the `DBType` enum.
/// 3.  `constraints` – A vector of constraints on the column. Types of constraints are defined in the `Constraint` enum.
#[derive(Debug, PartialEq)]
pub struct TableColumn {
    pub column_name: String,
    pub column_type: DBType,
    pub constraints: Vec<Constraint>,
}

/// A column in the database can be any of these types. `Int` and `Bool` types have no additional info, while the `Varchar(n)` type has an additional argument – the length of the string. Adding a type, such as `DECIMAL(n, m)` is boiled down to adding tokens for that type, parsing that type and adding it to this enum.
#[derive(Debug, PartialEq)]
pub enum DBType {
    Int,
    Varchar(usize),
    Bool,
}

/// A column can be limited to a domain of values, which is defined by constraints on that column. `PrimaryKey` and `NotNull` constraints have no additional info, while the `Check` constraints has an additional argument – the expression which every table row must satisfy.
#[derive(Debug, PartialEq)]
pub enum Constraint {
    NotNull,
    PrimaryKey,
    ForeignKey {
        column: String,
        referenced_table: String,
        referenced_column: String,
    },
    Check(Expression)
}

/// Binary and unary operators are defined as enums, where each enumeration constant represents one operator. Binary and unary operators are defined separately because a `-` (minus), for example can be in a binary operation: `5 - 4`, as well as in a unary operation: `-2`. `Asc` and `Desc` are `ORDER BY` operators that have the lowest operator precedence in any expression. While both unary and binary operators may be the exact same as tokens that represent them, it is important to make a distinction between them, as they are used in different contexts.
#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

/// Binary and unary operators are defined as enums, where each enumeration constant represents one operator. Binary and unary operators are defined separately because a `-` (minus), for example can be in a binary operation: `5 - 4`, as well as in a unary operation: `-2`. `Asc` and `Desc` are `ORDER BY` operators that have the lowest operator precedence in any expression. While both unary and binary operators may be the exact same as tokens that represent them, it is important to make a distinction between them, as they are used in different contexts.
#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Not,
    Plus,
    Minus,
    Asc,
    Desc,
}

// Example manual implementations for Display traits.
// Automatic derivation of those traits can be done, but the actual printing
// will be the same as in Debug prints which is not useful
// when printing to the end user.

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperator::Minus => write!(f, "-"),
            UnaryOperator::Plus => write!(f, "+"),
            UnaryOperator::Desc => write!(f, "DESC"),
            UnaryOperator::Asc => write!(f, "ASC"),
            UnaryOperator::Not => write!(f, "NOT"),
        }
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::GreaterThanOrEqual => write!(f, ">="),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::LessThanOrEqual => write!(f, "<="),
            BinaryOperator::Equal => write!(f, "="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Minus => write!(f, "-"),
            BinaryOperator::Plus => write!(f, "+"),
            BinaryOperator::And => write!(f, "AND"),
            BinaryOperator::Or => write!(f, "OR"),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::BinaryOperation { left_operand, operator, right_operand } => {
                write!(f, "({:?} {:?} {:?})", left_operand, operator, right_operand)
            }
            Expression::UnaryOperation { operand, operator } => {
                write!(f, "({:?} {:?})", operator, operand)
            }
            Expression::Number(num) => write!(f, "{num}"),
            Expression::Identifier(iden) => write!(f, "{}", iden),
            Expression::String(str) => write!(f, "\"{}\"", str),
            Expression::Bool(b) => write!(f, "{}", b)
        }
    }
}