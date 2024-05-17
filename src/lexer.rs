use std::iter::Peekable;
use std::str::{Chars};
use crate::token::Token;

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.next_token();
        if tok == Token::EOF {
            None
        } else {
            Some(tok)
        }
    }
}


impl <'a> Lexer<'a> {
    pub fn new (input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
        }
    }
    
    pub fn read_char(&mut self) -> char {
        self.input.next().unwrap_or('\0')
    }
    
    pub fn peek_char(&mut self) -> Option<&char> {
        self.input.peek()
    }
   
    pub fn is_letter(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    } 
    
    pub fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }
   
    pub fn read_number(&mut self, c: char) -> Option<Token> {
        let mut num = String::from(c);
        while let Some(&c) = self.peek_char() {
            if Self::is_digit(c) {
                num.push(self.read_char());
            } else {
                break;
            }
        }
        num.parse().ok().map(Token::Int)
    }
    
    
    pub fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek_char() {
            if c.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }
    
    pub fn read_identifier(&mut self, c: char) -> Option<Token> {
        let mut ident = String::from(c);
        while let Some(&c) = self.peek_char() {
            if Self::is_letter(c) {
                ident.push(self.read_char());
            } else {
                break;
            }
        }
        Some(Token::from_ident(ident))
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let c = if let Some(c) = self.input.next() { c } else { return Token::EOF };
        match c {
            c if Self::is_letter(c) => self.read_identifier(c),
            c if Self::is_digit(c) => self.read_number(c),
            '=' => {
                if let Some('=') = self.peek_char() {
                    self.read_char();
                    Some(Token::Eq)
                } else {
                    Some(Token::Assign)
                }
            }
            '!' => {
                if let Some('=') = self.peek_char() {
                    self.read_char();
                    Some(Token::NotEq)
                } else {
                    Some(Token::Bang)
                }
            },
            '+' => Some(Token::Plus),
            ',' => Some(Token::Comma),
            ';' => Some(Token::Semicolon),
            '(' => Some(Token::LParen),
            ')' => Some(Token::RParen),
            '{' => Some(Token::LBrace),
            '}' => Some(Token::RBrace),
            '-' => Some(Token::Minus),
            '/' => Some(Token::Slash),
            '*' => Some(Token::Asterisk),
            '<' => Some(Token::Lt),
            '>' => Some(Token::Gt),
            '\0' => Some(Token::EOF),
            _ => None,
        }.unwrap_or(Token::Illegal)
    }
}

mod tests {
    use crate::lexer::Lexer;
    use crate::token::Token;

    #[test]
    fn test_next_token() {
        let input = "=+(){},;";
        let tests = vec![
            Token::Assign,
            Token::Plus,
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::RBrace,
            Token::Comma,
            Token::Semicolon,
            Token::EOF,
        ];
        let mut lexer = Lexer::new(input);

        for tt in tests {
            let tok = lexer.next_token();
            assert_eq!(tok, tt);
        }
    }
    
    #[test]
    fn test_next_token_monkey_code() {
        let input = r#"let five = 5;
        let ten = 10;
        
        let add = fn(x, y) {
            x + y;
        };
        
        let result = add(five, ten);
        !-/*5;
        5 < 10 > 5;

        if (5 < 10) {
            return true;
        } else {
            return false;
        }
        
        10 == 10;
        10 != 9;
        "#;
        
        let tests = vec![
            Token::Let,
            Token::Ident("five".to_string()),
            Token::Assign,
            Token::Int(5),
            Token::Semicolon,
            Token::Let,
            Token::Ident("ten".to_string()),
            Token::Assign,
            Token::Int(10),
            Token::Semicolon,
            Token::Let,
            Token::Ident("add".to_string()),
            Token::Assign,
            Token::Function,
            Token::LParen,
            Token::Ident("x".to_string()),
            Token::Comma,
            Token::Ident("y".to_string()),
            Token::RParen,
            Token::LBrace,
            Token::Ident("x".to_string()),
            Token::Plus,
            Token::Ident("y".to_string()),
            Token::Semicolon,
            Token::RBrace,
            Token::Semicolon,
            Token::Let,
            Token::Ident("result".to_string()),
            Token::Assign,
            Token::Ident("add".to_string()),
            Token::LParen,
            Token::Ident("five".to_string()),
            Token::Comma,
            Token::Ident("ten".to_string()),
            Token::RParen,
            Token::Semicolon,
            Token::Bang,
            Token::Minus,
            Token::Slash,
            Token::Asterisk,
            Token::Int(5),
            Token::Semicolon,
            Token::Int(5),
            Token::Lt,
            Token::Int(10),
            Token::Gt,
            Token::Int(5),
            Token::Semicolon,
            Token::If,
            Token::LParen,
            Token::Int(5),
            Token::Lt,
            Token::Int(10),
            Token::RParen,
            Token::LBrace,
            Token::Return,
            Token::Bool(true),
            Token::Semicolon,
            Token::RBrace,
            Token::Else,
            Token::LBrace,
            Token::Return,
            Token::Bool(false),
            Token::Semicolon,
            Token::RBrace,
            Token::Int(10),
            Token::Eq,
            Token::Int(10),
            Token::Semicolon,
            Token::Int(10),
            Token::NotEq,
            Token::Int(9),
            Token::Semicolon,
            Token::EOF,
        ];
        let mut lexer = Lexer::new(input);

        for tt in tests {
            let tok = lexer.next_token();
            assert_eq!(tok, tt);
        }
    }
}