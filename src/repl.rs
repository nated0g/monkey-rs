use std::io::Write;
use crate::lexer::Lexer;

const PROMPT: &str = ">> ";

pub fn start() {
    let mut input = String::new();
    loop {
        print!("{}", PROMPT);
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        let l = Lexer::new(&input);
        
        for tok in l {
            println!("{:?}", tok);
        }
        input.clear();
    }
}
