extern crate regex;
extern crate clap;
extern crate libc;

use std::process;
use std::result;
use std::fs::File;
use clap::{Arg, App, ArgMatches};
use std::io::{BufReader,BufRead, stdin, Read};
use regex::Regex;

fn main() {
    reset_sigpipe();
    let matches = parse_args();
    match run(matches) {
        Ok(0) => process::exit(1),
        Ok(_) => process::exit(0),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
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
    fn execute(&self, s: String) -> result::Result<String, String>{
        match self.cmd {
            's' => {
                if self.x.cmd_subst.regex.is_match(&s){
                    Ok(self.x.cmd_subst.regex.replace_all(&s, &*self.x.cmd_subst.replacements[0]).to_string())
                } else { Ok(s) }
            },
            _ => Err("Fuck you".to_string())
        }
    }
}

fn match_slash<'a>(command: &'a str, index: usize, slash: char) -> Option<&'a str>{
    let mut chars = command[index..].char_indices();
    loop {
        match chars.next() {
            Some((i, item)) if item == slash => {
                match i {
                    0 => return None,
                    _ => return Some(&command[index..i + index])
                }
            },
            Some(_) => continue,
            None => break
        }
    };
    None
}

fn compile(command_str: String) -> result::Result<Command, String> {
    let mut chars = command_str.char_indices();
    let a : String = "hi".to_string();
    match chars.next() {
        Some((_, 's')) => {
            let (i, slash) = chars.next().unwrap();
            let raw_regex = match_slash(&command_str, i+1, slash).unwrap();
            let re = Regex::new(raw_regex).unwrap();
            let replacement = vec![match_slash(&command_str, raw_regex.len()+3, slash).unwrap().to_string()];
            let subst = Box::new(Subst { regex: re, replacements: replacement });
            return Ok(Command { cmd: 's', x: CommandData { cmd_subst: subst } });
        },
        _ => return Err("Not implemented or something".to_string())
    }
}

fn exec_per_line<T: BufRead>(cmd: Command, reader: T) -> result::Result<u64, String> {
    let mut matches = 0;
    for maybe_line in reader.lines() {
        if let Ok(line) = maybe_line {
            println!("{}", cmd.execute(line)?);
            matches += 1;
        }
    }
    Ok(matches)

}

fn run(args: ArgMatches) -> result::Result<u64, String> {
    let cmd = args.value_of("command").unwrap();
    let cmd = compile(cmd.to_string())?;
    match args.value_of("FILE"){
        Some(filename) => exec_per_line(cmd, BufReader::new(File::open(filename).unwrap())),
        None => exec_per_line(cmd, BufReader::new(stdin()))
    }
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
    use super::match_slash;
    use std::process;
    #[test]
    fn test_match_slashes() {
        assert_eq!(Some(r"hi"), match_slash(r"s/hi/there/g", 2, '/'));
    }

    #[test]
    fn test_indexed_on_slash() {
        assert_eq!(None, match_slash(r"s/hi/there/g", 1, '/'));

    }
}