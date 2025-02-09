// Simple Hangman Program
// User gets five incorrect guesses
// Word chosen randomly from words.txt
// Inspiration from: https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html
// This assignment will introduce you to some fundamental syntax in Rust:
// - variable declaration
// - string manipulation
// - conditional statements
// - loops
// - vectors
// - files
// - user input
// We've tried to limit/hide Rust's quirks since we'll discuss those details
// more in depth in the coming lectures.
extern crate rand;
use rand::Rng;
use std::fs;
use std::io;
use std::io::Write;
use std::iter::FromIterator;

const NUM_INCORRECT_GUESSES: u32 = 5;
const WORDS_PATH: &str = "words.txt";

fn pick_a_random_word() -> String {
    let file_string = fs::read_to_string(WORDS_PATH).expect("Unable to read file.");
    let words: Vec<&str> = file_string.split('\n').collect();
    String::from(words[rand::thread_rng().gen_range(0, words.len())].trim())
}

fn main() {
    let secret_word = pick_a_random_word();
    // Note: given what you know about Rust so far, it's easier to pull characters out of a
    // vector than it is to pull them out of a string. You can get the ith character of
    // secret_word by doing secret_word_chars[i].
    let secret_word_chars: Vec<char> = secret_word.chars().collect();
    // Uncomment for debugging:
    println!("random word: {}", secret_word);

    // Your code here! :)
    let mut guessed_word: String = "-".repeat(secret_word.len());
    let mut guessed_chars: Vec<char> = Vec::new();
    let mut incorrect_guesses: u32 = 0;

    println!("Welcome to CS110L Hangman!");

    while incorrect_guesses < NUM_INCORRECT_GUESSES && guessed_word != secret_word {
        println!("The word so far is {}", guessed_word);
        println!(
            "You have guessed the following letters: {}",
            String::from_iter(guessed_chars.iter())
        );
        println!(
            "You have {} guesses left",
            NUM_INCORRECT_GUESSES - incorrect_guesses
        );
        print!("Please guess a letter: ");
        io::stdout().flush().expect("Error flushing stdout.");
        let mut guess_line = String::new();
        io::stdin()
            .read_line(&mut guess_line)
            .expect("Error reading line.");
        let guess_char: char = guess_line.trim().chars().next().unwrap();
        guessed_chars.push(guess_char);

        if secret_word_chars.contains(&guess_char) {
            for i in 0..secret_word.len() {
                if secret_word_chars[i] == guess_char {
                    guessed_word.replace_range(i..=i, &guess_char.to_string());
                    // keep doing this until all occurrences of guess_char are replaced
                }
            }
        } else {
            println!("Sorry, that letter is not in the word");
            incorrect_guesses += 1;
        }

        println!();
    }

    if guessed_word == secret_word {
        println!(
            "Congratulations you guessed the secret word: {}!",
            secret_word
        );
    } else {
        println!("Sorry, you ran out of guesses!");
    }
}
