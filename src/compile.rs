extern crate regex;
extern crate std;
extern crate nom;
use regex::Regex;
use std::collections::HashSet;
use {SedCmd, Subst};
use {Addr, AddrState, NumBound, Bound, RegexBound, NoBound};
use std::any::Any;
use std::str;
use std::str::FromStr;

use nom::digit;

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

named!(num_bound<&str, Box<Bound> >,
    do_parse!(
        val: digit >>
        ({
            Box::new(NumBound{num:u64::from_str(val).unwrap()})
        })
    )
);

named!(regex_bound<&str, Box<Bound> >,
    do_parse!(
        slash: alt!(preceded!(tag!("\\"), take!(1)) | tag!("/")) >>
        regex_str: take_until_and_consume1!(slash) >>
        ({
            Box::new(RegexBound{regex: Regex::new(regex_str).unwrap()})
        })
    )
);

named!(two_bounds<&str, Addr>,
    do_parse!(
        start: alt!(num_bound | regex_bound) >>
        end: preceded!(tag!(","), alt!(num_bound | regex_bound)) >>
        ({
            Addr::new2(start, end)
        })
    )
);

named!(one_bound<&str, Addr>,
    do_parse!(
        start: alt!(num_bound | regex_bound) >>
        ({
            Addr::new1(start)
        })
    )
);

named!(addr<&str, Addr>,
    alt!(
        two_bounds |
        one_bound |
        value!(Addr::new0())
    )
);

#[test]
fn parse_num_bound() {
    let bound_box : Box<Bound> = num_bound("33 abc").unwrap().1;
    let b: &NumBound = match bound_box.as_any().downcast_ref::<NumBound>() {
        Some(b) => b,
        None => panic!("&a isn't a B!")
    };
    assert_eq!(b.num, 33);
}

#[test]
fn parse_regex_bound() {
    let bound_box : Box<Bound> = regex_bound("/hi/ abc").unwrap().1;
    let b: &RegexBound= match bound_box.as_any().downcast_ref::<RegexBound>() {
        Some(b) => b,
        None => panic!("&a isn't a B!")
    };
    assert_eq!(*b, RegexBound{regex:Regex::new("hi").unwrap()});
}

#[test]
fn parse_no_addr() {
    let singleton = NoBound{};
    let start_box: Box<Bound> = addr("abc").unwrap().1.start;
    let end_box: Box<Bound> = addr("abc").unwrap().1.end;
    assert_eq!(*start_box.as_any().downcast_ref::<NoBound>().unwrap(), singleton);
    assert_eq!(*end_box.as_any().downcast_ref::<NoBound>().unwrap(), singleton);
}

#[test]
fn parse_one_addr() {
    let singleton = NoBound{};
    let start_box: Box<Bound> = addr("115abc").unwrap().1.start;
    let end_box: Box<Bound> = addr("abc").unwrap().1.end;
    assert_eq!((*start_box.as_any().downcast_ref::<NumBound>().unwrap()).num, 115);
    assert_eq!(*end_box.as_any().downcast_ref::<NoBound>().unwrap(), singleton);
}


