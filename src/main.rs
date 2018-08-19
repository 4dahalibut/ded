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
use std::collections::HashSet;
use compile::compile;
use std::any::Any;

mod compile;

struct Addr {
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
}

#[derive(Debug, PartialEq)]
enum AddrState {
    Closed, Open
}

pub struct Subst {
    addr : Addr,
    regex : Regex,
    options: u8,
    replacements : Vec<String>
}

impl Subst {
    fn new(regex : Regex , replacements : Vec<String>) -> Subst {
        Subst{addr : Addr::new0(), regex, replacements, options : 0}
    }
}

#[derive(Copy, Clone)]
enum SubstType {
    Global, Print, Eval
}

trait SedCmd {
    fn execute(&mut self, s: &mut String);
}

trait Bound {
    fn matches(&mut self, linenum: u64, line_contents: String) -> bool;
    fn as_any(&self) -> &Any;
}


#[derive(Debug, PartialEq)]
pub struct NoBound {}
impl Bound for NoBound {
    fn matches(&mut self, linenum: u64, line_contents: String) -> bool{
        true
    }
    fn as_any(&self) -> &Any {
        self
    }
}

#[derive(Debug, PartialEq)]
pub struct NumBound {num: u64}
impl Bound for NumBound {
    fn matches(&mut self, linenum: u64, line_contents: String) -> bool{
        self.num == linenum
    }
    fn as_any(&self) -> &Any {
        self
    }

}

#[derive(Debug)]
pub struct RegexBound {regex: Regex}
impl Bound for RegexBound {
    fn matches(&mut self, linenum: u64, line_contents: String) -> bool{
        self.regex.is_match(line_contents.as_str())
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

fn main() {
    reset_sigpipe();
    run(parse_args());
}

impl SedCmd for Subst {
    fn execute(&mut self, s: &mut String){
        if self.regex.is_match(&s) {
            *s = self.regex.replace_all(s, &*self.replacements[0]).to_string();
            self.options += SubstType::Global as u8;
        }
    }
}

fn execute<T: BufRead>(cmds: &mut Vec<Box<SedCmd>>, mut reader: T) {
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap() != 0 {
        for cmd in cmds.iter_mut() {
            cmd.execute(&mut buf);
        };
        print!("{}", buf);
        buf.clear();
    };
}

fn run(args: ArgMatches)  {
    let cmd = args.value_of("command").unwrap();
    let input = match args.value_of("FILE") {
        Some(filename) => Box::new(BufReader::new(File::open(filename).unwrap())) as Box<BufRead>,
        None => Box::new(BufReader::new(stdin())) as Box<BufRead>
    };
    let mut cmds = compile(cmd.to_string());
    execute(&mut cmds, input);
}

fn parse_args() -> ArgMatches<'static> {
    App::new("Ded - Killin Sed")
        .about("Doesnt do much of anythin")
        .arg(Arg::with_name("command")
            .short("e")
            .long("expression")
            .value_name("command")
            .help("Append the editing commands specified by the command argument to the list of commands.")
            .takes_value(true)
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