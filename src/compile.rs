extern crate regex;
extern crate std;
use regex::Regex;
use std::collections::HashSet;
use {SedCmd, Subst};

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

pub fn compile(raw_commands: String) -> Vec<Box<SedCmd>> {
    let mut commands : Vec<Box<SedCmd>> = Vec::new();
    for command in raw_commands.split(';') {
        let mut chars = command.chars();
        match chars.next() {
            Some('s') => {
                let slash = chars.next().unwrap();
                let rest: Vec<String> = split_at_slash(chars.as_str(), slash);
                assert_eq!(rest.len(), 3);
                let re = Regex::new(&rest[0]).unwrap();
                let replacement = vec![rest[1].to_string()];
                commands.push(Box::new(Subst::new(re, replacement)));
            },
            _ => panic!("Not implemented or something".to_string())
        }
    }
    commands
}

