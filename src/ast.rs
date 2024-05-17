use std::fmt;
use std::fmt::{Display, Formatter};

use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let{ ident: Identifier, value: Expression },
    Return{ value: Expression },
    Expression{ value: Expression },
    Block{ statements: Vec<Statement> },
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Statement::Let { ident, value } => write!(f, "{} {} = {};", Token::Let, ident.value, value),
            Statement::Return { value } => write!(f, "{} {};", Token::Return, value),
            Statement::Expression { value } => write!(f, "{}", value),
            Statement::Block { statements } => {
                write!(f, "{{")?;
                for statement in statements {
                    write!(f, "{}", statement)?;
                }
                write!(f, "}}")?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    pub value: String,
}

impl Identifier {
    pub fn try_from_token(value: &Token) -> Option<Self> {
        match value {
            Token::Ident(ident) => Some(Self {
                value: ident.to_string(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(i64),
    Boolean(bool),
    Prefix {
        operator: Token,
        right: Box<Expression>,
    },
    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        consequence: Box<Statement>,
        alternative: Option<Box<Statement>>,
    },
    
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Identifier(ident) => write!(f, "{}", ident.value),
            Expression::IntegerLiteral(value) => write!(f, "{}", value),
            Expression::Boolean(value) => write!(f, "{}", value),
            Expression::Prefix { operator, right } => write!(f, "({}{})", operator, right),
            Expression::Infix { left, operator, right } => write!(f, "({} {} {})", left, operator, right),
            Expression::If { condition, consequence, alternative } => {
                write!(f, "{} {} ", Token::If, condition)?;
                write!(f, "{}", consequence)?;
                if let Some(alt) = alternative {
                    write!(f, " {} {}", Token::Else, alt)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub struct Program { 
    pub statements: Vec<Statement>,
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for statement in &self.statements {
            write!(f, "{}", statement)?;
        }
        Ok(())
    }
}

impl Program {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
}


#[cfg(test)]
mod tests {
    use super::{Expression, Identifier, Program, Statement};
    #[test]
    fn test_display_program() {
        let program = Program {
            statements: vec![
                Statement::Let {
                    ident: Identifier { value: "myVar".to_string() },
                    value: Expression::IntegerLiteral(5),
                },
                Statement::Let {
                    ident: Identifier { value: "anotherVar".to_string() },
                    value: Expression::Identifier(Identifier { value: "myVar".to_string() }),
                },
                Statement::Return {
                    value: Expression::Identifier(Identifier { value: "anotherVar".to_string() }),
                },
            ],
        };
        assert_eq!(format!("{}", program), "let myVar = 5;\nlet anotherVar = myVar;\nreturn anotherVar;\n");
    }
}