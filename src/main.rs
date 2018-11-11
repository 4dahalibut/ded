#![feature(get_type_id)]
#[macro_use]
extern crate nom;

extern crate clap;
extern crate libc;
extern crate regex;

use clap::{App, Arg, ArgMatches};
use crate::compile::toplevelparser;
use crate::functions::SedCmd;
use regex::Regex;
use std::any::Any;
use std::cell::RefCell;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::rc::Rc;

mod compile;
mod functions;

pub struct Addr {
    num_bounds: NumBounds,
    start: Option<Box<Bound>>,
    end: Option<Box<Bound>>,
    state: Rc<RefCell<AddrState>>,
}

impl Addr {
    fn new0() -> Addr {
        Addr {
            num_bounds: NumBounds::ZERO,
            start: None,
            end: None,
            state: Rc::new(RefCell::new(AddrState::Unborn)),
        }
    }

    fn new1(start: Box<Bound>) -> Addr {
        Addr {
            num_bounds: NumBounds::ONE,
            start: Some(start),
            end: None,
            state: Rc::new(RefCell::new(AddrState::Unborn)),
        }
    }

    fn new2(start: Box<Bound>, end: Box<Bound>) -> Addr {
        Addr {
            num_bounds: NumBounds::TWO,
            start: Some(start),
            end: Some(end),
            state: Rc::new(RefCell::new(AddrState::Unborn)),
        }
    }

    fn matches(&self, linenum: u64, line_contents: String) -> bool {
        let copystate = self.state.borrow().clone();
        match copystate {
            AddrState::Unborn => match &self.start {
                Some(bound) => {
                    if bound.matches(linenum, &line_contents) {
                        if self.num_bounds == NumBounds::TWO {
                            self.state.replace(AddrState::Open);
                        }
                        true
                    } else {
                        false
                    }
                }
                None => true,
            },
            AddrState::Open => match &self.end {
                Some(bound) => {
                    if bound.matches(linenum, &line_contents) {
                        self.state.replace(AddrState::Closed);
                        true
                    } else {
                        true
                    }
                }
                None => true,
            },
            AddrState::Closed => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum AddrState {
    Unborn,
    Closed,
    Open,
}

#[derive(Debug, PartialEq, Clone)]
enum NumBounds {
    ZERO,
    ONE,
    TWO,
}

trait Bound {
    fn matches(&self, linenum: u64, line_contents: &str) -> bool;
    fn as_any(&self) -> &Any;
}

#[derive(Debug, PartialEq)]
pub struct NumBound {
    num: u64,
}
impl Bound for NumBound {
    fn matches(&self, linenum: u64, _line_contents: &str) -> bool {
        self.num == linenum
    }
    fn as_any(&self) -> &Any {
        self
    }
}

#[derive(Debug)]
pub struct RegexBound {
    regex: Regex,
}
impl Bound for RegexBound {
    fn matches(&self, _linenum: u64, line_contents: &str) -> bool {
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

fn execute<T: BufRead>(cmd: &mut Box<SedCmd>, mut reader: T) {
    let mut pattern_space = String::new();
    let mut hold_space = String::new();
    let mut linenum = 1;
    while reader.read_line(&mut pattern_space).unwrap() != 0 {
        if !pattern_space.ends_with("\n") {
            pattern_space.push('\n');
        }
        cmd.execute(linenum, &mut hold_space, &mut pattern_space);
        print!("{}", pattern_space);
        pattern_space.clear();
        linenum += 1;
    }
}

fn main() {
    reset_sigpipe();
    run(parse_args());
}

fn run(args: ArgMatches) {
    let raw_command_text = args.value_of("command").unwrap();
    let input = match args.value_of("FILE") {
        Some(filename) => Box::new(BufReader::new(File::open(filename).unwrap())) as Box<BufRead>,
        None => Box::new(BufReader::new(stdin())) as Box<BufRead>,
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
