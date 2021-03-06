extern crate nom;
extern crate regex;

use crate::addr::Addr;
use regex::Regex;

pub trait SedCmd {
    fn execute(&self, linenum: u64, hold_space: &mut String, pattern_space: &mut String);
}

#[derive(Debug)]
pub struct Subst {
    pub find: Regex,
    pub replace: String,
    global: bool,
    print: bool,
    eval: bool,
}

impl Subst {
    pub fn new(regex: Regex, replace: String) -> Subst {
        Subst {
            find: regex,
            replace,
            global: false,
            print: false,
            eval: false,
        }
    }
}

impl SedCmd for Subst {
    fn execute(&self, _linenum: u64, _hold_space: &mut String, pattern_space: &mut String) {
        if self.find.is_match(&pattern_space) {
            *pattern_space = self
                .find
                .replace_all(pattern_space, &*self.replace)
                .to_string();
        }
    }
}

pub struct Append {
    text: String,
}

impl SedCmd for Append {
    fn execute(&self, _linenum: u64, _hold_space: &mut String, pattern_space: &mut String) {
        *pattern_space += &self.text;
    }
}

pub struct AppendHold {}

impl SedCmd for AppendHold {
    fn execute(&self, _linenum: u64, hold_space: &mut String, pattern_space: &mut String) {
        *pattern_space += "\n";
        *pattern_space += hold_space;
    }
}

pub struct MoreSedCmds {
    pub cmds: Vec<(Addr, Box<SedCmd>)>,
}

impl SedCmd for MoreSedCmds {
    fn execute(&self, linenum: u64, hold_space: &mut String, pattern_space: &mut String) {
        for (addr, cmd) in &self.cmds {
            if addr.matches(linenum, pattern_space.to_string()) {
                cmd.execute(linenum, hold_space, pattern_space)
            }
        }
    }
}
