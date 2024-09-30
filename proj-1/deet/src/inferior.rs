use crate::dwarf_data::DwarfData;
use ::std::collections::HashMap;
use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::mem::size_of;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

pub struct Inferior {
    child: Child,
    bps: HashMap<usize, Option<u8>>,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>, breakpoints: &Vec<usize>) -> Option<Inferior> {
        let mut cmd = Command::new(target);
        cmd.args(args);
        unsafe {
            cmd.pre_exec(child_traceme);
        }
        match cmd.spawn() {
            Ok(child) => {
                let child_pid = nix::unistd::Pid::from_raw(child.id() as i32);
                match waitpid(child_pid, None).ok()? {
                    WaitStatus::Stopped(_pid, _signal) => {
                        let mut inferior = Inferior {
                            child,
                            bps: HashMap::new(),
                        };
                        for breakpoint in breakpoints.iter() {
                            let orig_byte = inferior
                                .write_byte(*breakpoint, 0xcc)
                                .expect(&format!("Failed to set breakpoint at {}", breakpoint));
                            inferior.bps.insert(*breakpoint, Some(orig_byte));
                        }
                        Some(inferior)
                    }
                    _ => None,
                }
            }
            Err(_) => None,
        }
    }

    /// Wakes up the inferior and waits until it stops or terminates.
    pub fn wake_and_wait(&mut self, breakpoints: &Vec<usize>) -> Result<Status, nix::Error> {
        // New breakpoints might be added before continuing
        for breakpoint in breakpoints.iter() {
            let orig_byte = self
                .write_byte(*breakpoint, 0xcc)
                .expect(&format!("Failed to set breakpoint at {}", breakpoint));
            self.bps.insert(*breakpoint, Some(orig_byte));
        }

        // where i am
        let mut regs = ptrace::getregs(self.pid())?;
        let instruction_ptr = regs.rip as usize;

        // if inferior stopped at a breakpoint
        if let Some((&addr, &Some(orig_byte))) = self.bps.get_key_value(&(instruction_ptr - 1)) {
            // restore the first byte of the instruction
            let _ = self.write_byte(addr, orig_byte);
            // set %rip = %rip - 1 to rewind the instruction pointer
            regs.rip = (instruction_ptr - 1) as u64; // `usize`?
            ptrace::setregs(self.pid(), regs)?;
            // ptrace::step to go to next instruction
            ptrace::step(self.pid(), None)?;
            // wait for inferior to stop due to SIGTRAP
            match self.wait(None).unwrap() {
                Status::Exited(exit_code) => return Ok(Status::Exited(exit_code)),
                Status::Signaled(signal) => return Ok(Status::Signaled(signal)),
                Status::Stopped(_, _) => {
                    self.write_byte(instruction_ptr - 1, 0xcc);
                }
            }
        }

        ptrace::cont(self.pid(), None)?;
        self.wait(None)
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }

    /// Kills this inferior.
    pub fn kill(&mut self) -> Result<(), std::io::Error> {
        self.child.kill()
    }

    pub fn print_backtrace(&self, dwarf_data: &DwarfData) -> Result<(), nix::Error> {
        let regs = ptrace::getregs(self.pid())?;
        let mut instruction_ptr = regs.rip as usize;
        let mut base_ptr = regs.rbp as usize;

        loop {
            let line_number = dwarf_data.get_line_from_addr(instruction_ptr).unwrap();
            let function_name = dwarf_data.get_function_from_addr(instruction_ptr).unwrap();
            println!("{} ({})", function_name, line_number);

            if function_name == "main" {
                break;
            }

            instruction_ptr =
                ptrace::read(self.pid(), (base_ptr + 8) as ptrace::AddressType)? as usize;
            base_ptr = ptrace::read(self.pid(), base_ptr as ptrace::AddressType)? as usize;
        }

        Ok(())
    }

    fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        unsafe {
            ptrace::write(
                self.pid(),
                aligned_addr as ptrace::AddressType,
                updated_word as *mut std::ffi::c_void,
            )?;
        }
        Ok(orig_byte as u8)
    }
}
