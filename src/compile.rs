#![feature(trace_macros)]
extern crate regex;
extern crate std;
extern crate nom;
use regex::Regex;
use functions::{SedCmd, Subst, MoreSedCmds, AppendHold};
use {Addr, NumBound, Bound, RegexBound, NoBound};
use std::str;
use std::str::FromStr;

use nom::digit;


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
    let bound_box : Box<Bound> = num_bound("33 abc").unwrap().1;
    let b: &NumBound = match bound_box.as_any().downcast_ref::<NumBound>() {
        Some(b) => b,
        None => panic!("&a isn't a B!")
    };
    assert_eq!(b.num, 33);
}

#[test]
fn parse_regex_bound() {
    let bound_box : Box<Bound> = regex_bound("\\_hi_ abc").unwrap().1;
    let b: &RegexBound= match bound_box.as_any().downcast_ref::<RegexBound>() {
        Some(b) => b,
        None => panic!("&a isn't a B!")
    };
    assert_eq!(*b, RegexBound{regex:Regex::new("hi").unwrap()});
}

#[test]
fn parse_no_addr() {
    let singleton = NoBound{};
    let addr_to_test = addr("abc").unwrap();
    assert_eq!(addr_to_test.0, "abc");
    assert_eq!(*addr_to_test.1.start.as_any().downcast_ref::<NoBound>().unwrap(), singleton);
    assert_eq!(*addr_to_test.1.end.as_any().downcast_ref::<NoBound>().unwrap(), singleton);
}

#[test]
fn parse_one_addr() {
    let addr_to_test = addr("/wot/ abc").unwrap();
    assert_eq!(addr_to_test.0, "abc");
    assert_eq!((*addr_to_test.1.start.as_any().downcast_ref::<RegexBound>().unwrap()).regex.as_str(), "wot");
    assert_eq!(*addr_to_test.1.end.as_any().downcast_ref::<NoBound>().unwrap(), NoBound{});
}

#[test]
fn parse_two_addr() {
    let addr_to_test = addr("115,/end/ abc").unwrap();
    assert_eq!(addr_to_test.0, "abc");
    assert_eq!((*addr_to_test.1.start.as_any().downcast_ref::<NumBound>().unwrap()).num, 115);
    assert_eq!((*addr_to_test.1.end.as_any().downcast_ref::<RegexBound>().unwrap()).regex.as_str(), "end");
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
        modifier: take!(1) >>
        ({
            Box::new(Subst::new(Regex::new(find).unwrap(), replace.to_string()))
        })
    )
);

named!(pub toplevelparser<&str, Box<SedCmd> >,
    do_parse!(
        cmds: separated_list!(
            tag!(";"),
            tuple!(addr,
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
            )
        ) >>
        ({
            Box::new(MoreSedCmds{cmds})
        })
    )
);

#[test]
fn one_substitute_cmd() {
    let parsed = substitute(":this:that:g").unwrap();
    let subst = parsed.1.as_any().downcast_ref::<Subst>().unwrap();
    assert_eq!(subst.find.as_str(), "this");
    assert_eq!(subst.replace, "that");
    assert_eq!(parsed.0, "");
}

#[test]
fn compile_g() {
    let (addr, cmd) = toplevelparser("1G");
    assert_eq!((*addr.1.start.as_any().downcast_ref::<NumBound>().unwrap()).num, "1");
    let mut pattern_space = "this";
    let mut hold_space = "that";
    cmd.unwrap().execute(pattern_space, hold_space);
    assert_eq!(pattern_space, "this\nthat");

}
