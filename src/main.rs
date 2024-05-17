mod token;
mod lexer;
mod repl;
mod ast;
mod parser;

fn main() {
    // welcome the user
    println!("Welcome to the Monkey programming language REPL!");
    println!("Feel free to type in commands");
    repl::start();
}
