use crate::debugger_command::DebuggerCommand;
use crate::inferior::{Inferior, Status};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<(), FileHistory>,
    inferior: Option<Inferior>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<(), FileHistory>::new().expect("Failed to create Editor");
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Quit => {
                    self.clean();
                    return;
                }
                DebuggerCommand::Run(args) => {
                    self.clean();

                    if let Some(inferior) = Inferior::new(&self.target, &args) {
                        // Create the inferior
                        self.inferior = Some(inferior);

                        // milestone 1: make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        match self.inferior.as_mut().unwrap().wake_and_wait() {
                            Ok(status) => match status {
                                Status::Stopped(signal, _) => {
                                    println!("Child stopped (signal {})", signal.as_str())
                                }
                                Status::Exited(code) => {
                                    println!("Child exited (status {})", code);
                                }
                                Status::Signaled(signal) => {
                                    println!("Child signaled (signal {})", signal.as_str());
                                }
                            },
                            Err(_) => println!("Error waking up the inferior and waiting"),
                        }
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Continue => {
                    if self.inferior.is_none() {
                        println!("Inferior is not running");
                    } else {
                        self.inferior.as_mut().unwrap().wake_and_wait();
                    }
                }
                DebuggerCommand::Backtrace => {
                    self.inferior.as_mut().unwrap().print_backtrace();
                }
            }
        }
    }

    /// Kills any existing inferiors
    fn clean(&mut self) {
        if self.inferior.is_some() {
            let inferior_refmut = self.inferior.as_mut().unwrap();
            println!("Killing running inferior (pid {})", inferior_refmut.pid());
            match inferior_refmut.kill() {
                Ok(_) => println!("Killed"),
                Err(e) => println!("Failed to kill: {}", e),
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    let _ = self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
