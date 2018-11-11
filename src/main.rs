#![feature(get_type_id)]
extern crate clap;
extern crate libc;
#[macro_use]
extern crate nom;
extern crate regex;

use clap::{App, Arg, ArgMatches};
use crate::compile::toplevelparser;
use crate::functions::SedCmd;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};

mod addr;
mod compile;
mod functions;

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
    App::new("Ded - Rusty sed")
        .about("Reinventing a wheel")
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
