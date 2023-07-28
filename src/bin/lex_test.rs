use std::env;
use std::fs;
use turmeric::lex;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <file name>", args[0]);
        return;
    }
    let buf = fs::read(&args[1]).unwrap();
    let tokens = lex::lex(&buf[..]).unwrap();
    for tok in tokens {
        println!("{:?}", tok);
    }
}
