use crate::debugger_command::DebuggerCommand;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use crate::inferior::{Inferior, Status};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;

fn parse_address(addr: &str) -> Option<usize> {
    let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
        &addr[2..]
    } else {
        &addr
    };
    usize::from_str_radix(addr_without_0x, 16).ok()
}

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<(), FileHistory>,
    inferior: Option<Inferior>,
    dwarf_data: DwarfData,
    breakpoints: Vec<usize>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // Milestone 3: initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };

        // FOR TEST
        debug_data.print();

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<(), FileHistory>::new().expect("Failed to create Editor");
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            dwarf_data: debug_data,
            breakpoints: Vec::new(),
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

                    if let Some(inferior) = Inferior::new(&self.target, &args, &self.breakpoints) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        self.wake_and_wait();
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Continue => {
                    if self.inferior.is_none() {
                        println!("Inferior is not running");
                    } else {
                        self.wake_and_wait();
                    }
                }
                DebuggerCommand::Backtrace => {
                    let _ = self
                        .inferior
                        .as_mut()
                        .unwrap()
                        .print_backtrace(&self.dwarf_data);
                }
                DebuggerCommand::Break(arg) => {
                    if arg.starts_with("*") {
                        let addr_without_0x =
                            parse_address(&arg[1..]).expect("Failed to parse the address");
                        println!(
                            "Set breakpoint {} at {:#x}",
                            self.breakpoints.len(),
                            addr_without_0x
                        );
                        self.breakpoints.push(addr_without_0x);
                    }
                }
            }
        }
    }

    fn wake_and_wait(&mut self) {
        // Milestone 1: make the inferior run
        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
        // to the Inferior object
        match self.inferior.as_mut().unwrap().wake_and_wait() {
            Ok(status) => match status {
                Status::Stopped(signal, instruction_ptr) => {
                    println!("Child stopped (signal {})", signal.as_str());
                    let line_number = self.dwarf_data.get_line_from_addr(instruction_ptr).unwrap();
                    println!("Stopped at {}", line_number);
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
