use std::iter::Peekable;
use crate::ast::{Expression, Identifier, Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;
use anyhow::{Result, Error};

/// Precedence levels for operators
/// The order of the variants in the definition of Precedence is important
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
}

impl Precedence {
    pub fn from_token(tok: &Token) -> Self {
        match tok {
            Token::Eq | Token::NotEq => Precedence::Equals,
            Token::Lt | Token::Gt => Precedence::LessGreater,
            Token::Plus | Token::Minus => Precedence::Sum,
            Token::Asterisk | Token::Slash => Precedence::Product,
            _ => Precedence::Lowest,
        }
    }
}

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer.peekable(),
        }
    }
    
    pub fn parse_let_statement(&mut self) -> Result<Statement> {
        let ident = self.try_consume_ident()?;
        self.try_consume_token(Token::Assign)?;
        for tok in self.lexer.by_ref() {
            // TODO: Parse expression
            if tok == Token::Semicolon {
                break;
            }
        }
        Ok(Statement::Let { ident, value: Expression::IntegerLiteral(0) })
    }

    pub fn parse_return_statement(&mut self) -> Result<Statement> {
        for tok in self.lexer.by_ref() {
            // TODO: Parse expression
            if tok == Token::Semicolon {
                break;
            }
        }
        Ok(Statement::Return { value: Expression::IntegerLiteral(0) })
    }
    
    
    pub fn try_consume_token(&mut self, tok: Token) -> Result<Token> {
        match self.lexer.peek() {
            Some(t) => {
                if *t == tok {
                    self.lexer.next();
                    Ok(tok)
                } else {
                    Err(Error::msg(format!("Expected {:?}, got {:?}", tok, t)))
                }
            },
            _ => Err(Error::msg(format!("Expected {:?}, got EOF", tok)))
        }
    }
    
    pub fn try_consume_ident(&mut self) -> Result<Identifier> {
        match self.lexer.peek() {
            Some(tok) => {
                if let Some(ident) = Identifier::try_from_token(tok) {
                    self.lexer.next();
                    Ok(ident)
                } else {
                    Err(Error::msg("Expected identifier"))
                }
            },
            _ => Err(Error::msg("Expected identifier")),
        }
    }

    pub fn parse_expression_statement(&mut self) -> Result<Statement> {
        let expression = self.parse_expression(Precedence::Lowest)?;
        if let Some(tok) = self.lexer.peek() {
            if *tok == Token::Semicolon {
                self.lexer.next();
            }
        }
        Ok(Statement::Expression { value: expression })
    }
    
    pub fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression> {
        let operator = self.lexer.next().unwrap();
        let precedence = Precedence::from_token(&operator);
        let right = Box::new(self.parse_expression(precedence)?);
        Ok(Expression::Infix { left: Box::new(left), operator, right })
    }
    
    pub fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        let mut expr = match self.lexer.next() {
            Some(tok) => {
                match tok {
                    Token::Ident(ident) => Expression::Identifier(Identifier { value: ident }),
                    Token::Int(int) => Expression::IntegerLiteral(int),
                    Token::Bool(boolean) => Expression::Boolean(boolean),
                    Token::Bang | Token::Minus => {
                        let operator = tok;
                        let right = Box::new(self.parse_expression(Precedence::Prefix)?);
                        Expression::Prefix { operator, right }
                    },
                    Token::LParen => {
                        let expr = self.parse_expression(Precedence::Lowest)?;
                        match self.lexer.peek() {
                            Some(Token::RParen) => {
                                self.lexer.next();
                                expr
                            },
                            Some(tok) => return Err(Error::msg(format!("Expected RParen, found {:?}", tok))),
                            None => return Err(Error::msg("Expected RParen, found EOF")),
                        }
                        
                    },
                    Token::If => {
                        self.try_consume_token(Token::LParen)?; // consume LParen
                        let condition = Box::new(self.parse_expression(Precedence::Lowest)?);
                        self.try_consume_token(Token::RParen)?; // consume RParen
                        self.try_consume_token(Token::LBrace)?;
                        let consequence = Box::new(Statement::Expression { value: self.parse_expression(Precedence::Lowest)? });
                        let alternative = if let Some(Token::Else) = self.lexer.peek() {
                            self.lexer.next();
                            self.try_consume_token(Token::LBrace)?;
                            Some(Box::new(Statement::Expression { value: self.parse_expression(Precedence::Lowest)? }))
                        } else {
                            None
                        };
                        Expression::If { condition, consequence, alternative }
                    }
                    _ => return Err(Error::msg(format!("Unexpected token {:?}", tok))),
                }
            },
            _ => return Err(Error::msg("Unexpected EOF")),
        };
        
        while let Some(tok) = self.lexer.peek() {
            let peeked_precedence = Precedence::from_token(tok);
            if *tok != Token::Semicolon && precedence < peeked_precedence {
                expr = self.parse_infix_expression(expr)?;
            } else {
                break
            }
        }
        
        Ok(expr)
    }
    
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut program = Program::new();
        let mut errors: Vec<String> = Vec::new();
        
        while let Some(tok) = self.lexer.peek() {
            match tok {
                Token::Let => {
                    self.lexer.next();
                    match self.parse_let_statement() {
                        Ok(statement) => program.add_statement(statement),
                        Err(e) => errors.push(e.to_string()),
                    }
                },
                Token::Return => {
                    self.lexer.next();
                    match self.parse_return_statement() {
                        Ok(statement) => program.add_statement(statement),
                        Err(e) => errors.push(e.to_string()),
                    }
                },
                _ => {
                    match self.parse_expression_statement() {
                        Ok(statement) => program.add_statement(statement),
                        Err(e) => errors.push(e.to_string()),
                    }
                },
            }
        }
        if !errors.is_empty() {
            Err(Error::msg(format!("Parser error: {:?}", errors)))
        } else {
            Ok(program)
        }
    }
    
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use crate::ast::Statement;
    use super::*;

    #[test]
    fn test_print_program() {
        let input = "let x = 5; let y = 10; let foobar = 838383;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        println!("{}", program);
    }
    
    #[test]
    fn test_let_statements() {
        let input = "let x = 5; let y = 10; let foobar = 838383;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        
        assert_eq!(program.statements.len(), 3);
        
        let expected = ["x", "y", "foobar"];
        for (i, name) in expected.iter().enumerate() {
            test_let_statement(program.statements[i].clone(), name);
        }
    }
    
    fn test_let_statement(statement: Statement, name: &str) {
        match statement {
            Statement::Let { ident, value: _ } => assert_eq!(ident.value, name),
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_return_statements() {
        let input = "return 5; return 10; return 993322;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 3);

        for statement in program.statements {
            match statement {
                Statement::Return { value: _ } => (),
                _ => panic!("Expected Return statement"),
            }
        }

    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        match program.statements[0].clone() {
            Statement::Expression { value } => {
                match value {
                    Expression::Identifier(ident) => assert_eq!(ident.value, "foobar"),
                    _ => panic!("Expected Identifier expression"),
                }
            },
            _ => panic!("Expected Expression statement"),
        }
    }
    
    #[test]
    fn test_integer_expression() {
        let input = "5;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.statements.len(), 1);

        match program.statements[0].clone() {
            Statement::Expression { value } => {
                match value {
                    Expression::IntegerLiteral(int) => assert_eq!(int, 5),
                    _ => panic!("Expected IntegerLiteral expression"),
                }
            },
            _ => panic!("Expected Expression statement"),
        }
    }
    
    #[test]
    fn test_prefix_expression() {
        let test_cases = vec![
            ("!5;", "!", 5),
            ("-15;", "-", 15),
        ];
        for (input, operator, right_val) in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();
            
            assert_eq!(program.statements.len(), 1);
            
            match program.statements[0].clone() {
                Statement::Expression { value } => {
                    match value {
                        Expression::Prefix { operator: op, right } => {
                            assert_eq!(op.to_string(), operator);
                            assert_eq!(right.deref(), &Expression::IntegerLiteral(right_val));
                        },
                        _ => panic!("Expected Prefix expression"),
                    }
                },
                _ => panic!("Expected Expression statement"),
            }
        }
    }
    #[test]
    fn test_infix_expression() {
        let test_cases = vec![
            ("5 + 5;", 5, "+", 5),
            ("5 - 5;", 5, "-", 5),
            ("5 * 5;", 5, "*", 5),
            ("5 / 5;", 5, "/", 5),
            ("5 > 5;", 5, ">", 5),
            ("5 < 5;", 5, "<", 5),
            ("5 == 5;", 5, "==", 5),
            ("5 != 5;", 5, "!=", 5),
        ];
        for (input, left_val, operator, right_val) in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();
            
            assert_eq!(program.statements.len(), 1);
            
            match program.statements[0].clone() {
                Statement::Expression { value } => {
                    match value {
                        Expression::Infix { left, operator: op, right } => {
                            assert_eq!(left.deref(), &Expression::IntegerLiteral(left_val));
                            assert_eq!(op.to_string(), operator);
                            assert_eq!(right.deref(), &Expression::IntegerLiteral(right_val));
                        },
                        _ => panic!("Expected Infix expression"),
                    }
                },
                _ => panic!("Expected Expression statement"),
            }
        }
    }
    
    #[test]
    fn test_operator_precedence_parsing() {
        let test_cases = vec![
            ("-a * b;", "((-a) * b)"),
            ("!-a;", "(!(-a))"),
            ("a + b + c;", "((a + b) + c)"),
            ("a + b - c;", "((a + b) - c)"),
            ("a * b * c;", "((a * b) * c)"),
            ("a * b / c;", "((a * b) / c)"),
            ("a + b / c;", "(a + (b / c))"),
            ("a + b * c + d / e - f;", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4; -5 * 5;", "(3 + 4)((-5) * 5)"),
            ("5 > 4 == 3 < 4;", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4;", "((5 < 4) != (3 > 4))"),
            ("3 + 4 * 5 == 3 * 1 + 4 * 5;", "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))"),
            ("!true", "(!true)"),
            ("!false", "(!false)"),
            ("(5 + 5) * 2", "((5 + 5) * 2)")
        ];
        for (input, expected) in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();
            
            assert_eq!(format!("{}", program), expected);
        }
    }
   
    #[test]
    fn test_boolean_literal() {
let test_cases = vec![
            ("true;", true),
            ("false;", false),
        ];
        for (input, expected) in test_cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program().unwrap();
            
            assert_eq!(program.statements.len(), 1);
            
            match program.statements[0].clone() {
                Statement::Expression { value } => {
                    match value {
                        Expression::Boolean(boolean) => assert_eq!(boolean, expected),
                        _ => panic!("Expected Boolean expression"),
                    }
                },
                _ => panic!("Expected Expression statement"),
            }
        }
    }
    
}