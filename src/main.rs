#![feature(get_type_id)]
#[macro_use]
extern crate nom;

extern crate regex;
extern crate clap;
extern crate libc;

use std::fs::File;
use clap::{Arg, App, ArgMatches};
use std::io::{BufReader,BufRead, stdin};
use regex::Regex;
use compile::toplevelparser;
use std::any::Any;
use functions::{SedCmd};

mod compile;
mod functions;

pub struct Addr {
    start: Box<Bound>,
    end: Box<Bound>,
    state : AddrState,
    step: Option<i32>,
}

impl Addr {
    fn new0() -> Addr {
        Addr{start:Box::new(NoBound{}), end: Box::new(NoBound{}), state: AddrState::Closed, step: None}
    }

    fn new1(start: Box<Bound>) -> Addr {
        Addr{start, end: Box::new(NoBound{}), state: AddrState::Closed, step: None}
    }

    fn new2(start: Box<Bound>, end: Box<Bound>) -> Addr {
        Addr{start, end, state: AddrState::Closed, step: None}
    }

    fn matches(&mut self, linenum: u64, line_contents: String) -> bool {
        if self.state == AddrState::Unborn {
            if self.start.matches(linenum, &line_contents) {
                self.state = AddrState::Open;
                return true;
            }
        }
        if self.state == AddrState::Open {
            if self.end.matches(linenum, &line_contents) {
                self.state = AddrState::Closed;
                return true;
            }
        }
        false
    }
}

#[derive(Debug, PartialEq)]
enum AddrState {
    Unborn, Closed, Open
}

trait Bound {
    fn matches(&mut self, linenum: u64, line_contents: &str) -> bool;
    fn as_any(&self) -> &Any;
}


#[derive(Debug, PartialEq)]
pub struct NoBound {}
impl Bound for NoBound {
    fn matches(&mut self, linenum: u64, line_contents: &str) -> bool{
        true
    }
    fn as_any(&self) -> &Any {
        self
    }
}

#[derive(Debug, PartialEq)]
pub struct NumBound {num: u64}
impl Bound for NumBound {
    fn matches(&mut self, linenum: u64, line_contents: &str) -> bool{
        self.num == linenum
    }
    fn as_any(&self) -> &Any {
        self
    }

}

#[derive(Debug)]
pub struct RegexBound {regex: Regex}
impl Bound for RegexBound {
    fn matches(&mut self, linenum: u64, line_contents: &str) -> bool{
        self.regex.is_match(line_contents)
    }
    fn as_any(&self) -> &Any {
        self
    }

}

impl PartialEq for RegexBound {
    fn eq(&self, other: &RegexBound) -> bool {
        self.regex.as_str() == other.regex.as_str()
    }
}
//TODO: Write logic for address matching, add said gate into all execute methods

fn execute<T: BufRead>(cmd: &mut Box<SedCmd>, mut reader: T) {
    let mut pattern_space = String::new();
    let mut hold_space = String::new();
    let mut linenum = 0;
    while reader.read_line(&mut pattern_space).unwrap() != 0 {
        cmd.execute(linenum, &mut hold_space, &mut pattern_space);
        print!("{}", pattern_space);
        pattern_space.clear();
        linenum += 1;
    };
}

fn main() {
    reset_sigpipe();
    run(parse_args());
}

fn run(args: ArgMatches)  {
    let raw_command_text = args.value_of("command").unwrap();
    let input = match args.value_of("FILE") {
        Some(filename) => Box::new(BufReader::new(File::open(filename).unwrap())) as Box<BufRead>,
        None => Box::new(BufReader::new(stdin())) as Box<BufRead>
    };
    let ref mut cmd = toplevelparser(raw_command_text).unwrap().1;
    execute(cmd, input);
}

fn parse_args() -> ArgMatches<'static> {
    App::new("Ded - Killin Sed")
        .about("Doesnt do much of anythin")
        .arg(Arg::with_name("silent")
            .short("n")
            .help("By default, each line of input is echoed to the standard output after all \
            of the commands have been applied to it.  The -n option suppresses this behavior"))
        .arg(Arg::with_name("command")
            .short("e")
            .long("expression")
            .value_name("command")
            .help("Append the editing commands specified by the command argument to the list of commands.")
            .takes_value(true)
            .multiple(true)
            .required(true))
        .arg(Arg::with_name("FILE")
            .help("Sets the input file to use")
            .takes_value(true)
            .index(1)
            .required(false))
        .get_matches()
}

#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn reset_sigpipe() {
    // no-op
}

#[cfg(test)]
mod tests {
}
