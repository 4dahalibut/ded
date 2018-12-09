extern crate nom;
extern crate regex;
extern crate std;

use crate::addr::{Addr, Bound, NumBound, RegexBound};
use crate::functions::{AppendHold, MoreSedCmds, SedCmd, Subst};
use nom::digit;
use regex::Regex;
use std::str;
use std::str::FromStr;

named!(num_bound<&str, Box<Bound> >,
    do_parse!(
    //TODO: Also have $ do a thing
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
    ws!(alt!(
        two_bounds |
        one_bound |
        value!(Addr::new0())
    ))
);

#[test]
fn parse_num_bound() {
    assert!(num_bound("33 abc").unwrap().1.matches(33, "Whatever"));
    assert!(!num_bound("33 abc").unwrap().1.matches(32, "Whatever"));
}

#[test]
fn parse_regex_bound() {
    assert!(regex_bound("\\_hi_ abc").unwrap().1.matches(9000, "hi there"));
    assert!(!regex_bound("\\_hi_ abc").unwrap().1.matches(9000, "goodbye"));
}

#[test]
fn parse_no_addr() {
    let parse_result = addr("abc").unwrap();
    assert_eq!(parse_result.0, "abc");
    assert!(parse_result.1.start.is_none());
    assert!(parse_result.1.end.is_none());
}

#[test]
fn parse_one_addr() {
    let result = addr("/wot/ abc").unwrap();
    assert_eq!(result.0, "abc");
    let addr_to_test : Addr = result.1;
    let start_bound = addr_to_test.start.unwrap();
    assert!(start_bound.matches(3, "wot is this"));
    assert!(!start_bound.matches(3, "not is this"));
    assert!(addr_to_test.end.is_none());
}

#[test]
fn parse_two_addr() {
    let parse_result = addr("33,/wot/ abc").unwrap();
    assert_eq!(parse_result.0, "abc");
    let addr_to_test : Addr = parse_result.1;
    let start_bound = addr_to_test.start.unwrap();
    let end_bound = addr_to_test.end.unwrap();
    assert!(start_bound.matches(33, "Whatever"));
    assert!(!start_bound.matches(32, "Whatever"));
    assert!(end_bound.matches(3, "wot is this"));
    assert!(!end_bound.matches(3, "not is this"));
}

named!(append_hold<&str, Box<SedCmd> >,
    do_parse!(
        ({
            Box::new(AppendHold{})
        })
    )
);

named!(substitute<&str, Box<SedCmd> >,
    do_parse!(
        slash: take!(1) >>
        find: take_until_and_consume1!(slash) >>
        replace: take_until_and_consume1!(slash) >>
        _modifier: take!(1) >>
        ({
            Box::new(Subst::new(Regex::new(find).unwrap(), replace.to_string()))
        })
    )
);

named!(aaalt<&str, Box<SedCmd> >,
    alt!(
        delimited!(tag!("{"), toplevelparser, tag!("}")) |
//                    preceded!(tag!("a\\"), append) |
//                    preceded!(tag!("b"), branch) |
//                    preceded!(tag!("c\\"), change) |
//                    preceded!(tag!("d"), delete) |
//                    preceded!(tag!("D"), delete_til_newline) |
//                    preceded!(tag!("g"), replace_with_hold) |
        preceded!(tag!("G"), append_hold) |
//                    preceded!(tag!("h"), replace_hold) |
//                    preceded!(tag!("H"), add_to_hold) |
//                    preceded!(tag!("i\\"), insert) |
//                    preceded!(tag!("l"), write_unambiguously) |
//                    preceded!(tag!("n"), next) |
//                    preceded!(tag!("N"), next_join) |
//                    preceded!(tag!("p"), print) |
//                    preceded!(tag!("P"), print_til_newline) |
//                    preceded!(tag!("q"), quit) |
//                    preceded!(tag!("r"), read) |
        preceded!(tag!("s"), substitute)
//                    preceded!(tag!("t"), branch_conditional) |
//                    preceded!(tag!("w"), write) |
//                    preceded!(tag!("x"), swap) |
//                    preceded!(tag!("y"), string_subst) |
//                    preceded!(tag!("#"), comment) |
//                    preceded!(tag!(":"), tag)
    )

);

named!(wrapped_single<&str, Vec<(Addr, Box<SedCmd>)> >,
    do_parse!(
        cmd: tuple!(addr, aaalt) >>
        ({
            vec!(cmd)
        })
    )
);

named!(pub toplevelparser<&str, Box<SedCmd> >,
    do_parse!(
        cmds: dbg_dmp!(alt!(
            wrapped_single |
            separated_list!(
                tag!(";"),
                tuple!(addr, aaalt )
            ))
        ) >>
        ({
            Box::new(MoreSedCmds{cmds})
        })
    )
);

#[test]
fn one_substitute_cmd() {
    let parsed = substitute(":this:that:g").unwrap();
    assert_eq!(parsed.0, "");
    let ref subst : SedCmd = *parsed.1;
    let ref mut hold_space = "whatever dude".to_string();
    let ref mut pattern_space = "this is great".to_string();
    subst.execute(4, hold_space, pattern_space);
    assert_eq!(pattern_space, "that is great");
    assert_eq!(hold_space, "whatever dude");
}

#[test]
fn compile_g() {
    let ref cmd = *toplevelparser("1G").unwrap().1;
    let mut pattern_space = "this".to_string();
    let mut hold_space = "that".to_string();
    cmd.execute(1, &mut hold_space, &mut pattern_space);
    assert_eq!(pattern_space, "this\nthat");
}
