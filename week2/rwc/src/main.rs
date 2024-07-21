use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    let file = File::open(filename).expect("Error opening file");
    let buf_reader = io::BufReader::new(file);
    let lines = buf_reader.lines();
    let mut n_lines: usize = 0;
    let mut n_words: usize = 0;
    let mut n_chars: usize = 0;
    for line in lines {
        let line_content = line.unwrap();
        n_lines += 1;
        n_words += line_content.split_whitespace().count();
        n_chars += line_content.chars().count();
    }
    println!("{} {} {}", n_lines, n_words, n_chars);
}
