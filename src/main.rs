extern crate regex;
extern crate clap;
extern crate libc;

use std::fs::File;
use clap::{Arg, App, ArgMatches};
use std::io::{BufReader,BufRead, stdin};
use regex::Regex;

fn main() {
    reset_sigpipe();
    run(parse_args());
}

struct Subst{
    regex: Regex,
    replacements: Vec<String>,
}

struct CommandData{
    cmd_subst: Box<Subst>,
}

struct Command {
    cmd: char,
    x: CommandData,
}

impl Command {
    fn execute(&self, s: &mut String){
        match self.cmd {
            's' => {
                if self.x.cmd_subst.regex.is_match(&s){
                    *s = self.x.cmd_subst.regex.replace_all(s, &*self.x.cmd_subst.replacements[0]).to_string();
                }
            },
            _ => panic!("Undefined cmd {} ", self.cmd),
        };
    }
}

fn split_at_slash(command: &str, slash: char) -> Vec<String>{
    let mut chars = command.chars();
    let mut string_builder = "".to_string();
    let mut matched_strs : Vec<String> = Vec::new();
    loop {
        match chars.next() {
            Some(c) if c == slash && !string_builder.ends_with(r"\")=> {
                matched_strs.push(string_builder.clone());
                string_builder = "".to_string();
            },
            Some(c) => string_builder.push(c),
            None => break
        };
    };
    matched_strs.push(string_builder.clone());
    matched_strs
}

fn compile(raw_commands: String) -> Vec<Command> {
    let mut commands : Vec<Command> = Vec::new();
    for command in raw_commands.split(';') {
        let mut chars = command.chars();
        match chars.next() {
            Some('s') => {
                let slash = chars.next().unwrap();
                let rest: Vec<String> = split_at_slash(chars.as_str(), slash);
                assert_eq!(rest.len(), 3);
                let re = Regex::new(&rest[0]).unwrap();
                let replacement = vec![rest[1].to_string()];
                let subst = Box::new(Subst { regex: re, replacements: replacement });
                commands.push(Command { cmd: 's', x: CommandData { cmd_subst: subst } });
            },
            _ => panic!("Not implemented or something".to_string())
        }
    }
    commands
}

fn execute<T: BufRead>(cmds: Vec<Command>, mut reader: T) {
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap() != 0 {
        for cmd in cmds.iter() {
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
    let cmds = compile(cmd.to_string());
    execute(cmds, input);
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