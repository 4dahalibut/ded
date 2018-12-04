use regex::Regex;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Addr {
    num_bounds: u8,
    pub start: Option<Box<Bound>>,
    pub end: Option<Box<Bound>>,
    state: Rc<RefCell<AddrState>>,
}

impl Addr {
    pub fn new0() -> Addr {
        Addr {
            num_bounds: 0,
            start: None,
            end: None,
            state: Rc::new(RefCell::new(AddrState::Unborn)),
        }
    }

    pub fn new1(start: Box<Bound>) -> Addr {
        Addr {
            num_bounds: 1,
            start: Some(start),
            end: None,
            state: Rc::new(RefCell::new(AddrState::Unborn)),
        }
    }

    pub fn new2(start: Box<Bound>, end: Box<Bound>) -> Addr {
        Addr {
            num_bounds: 2,
            start: Some(start),
            end: Some(end),
            state: Rc::new(RefCell::new(AddrState::Unborn)),
        }
    }

    pub fn matches(&self, linenum: u64, line_contents: String) -> bool {
        let copystate = self.state.borrow().clone();
        match copystate {
            AddrState::Unborn => match &self.start {
                Some(bound) => {
                    if bound.matches(linenum, &line_contents) {
                        if self.num_bounds == 2 {
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

pub trait Bound {
    fn matches(&self, linenum: u64, line_contents: &str) -> bool;
    fn as_any(&self) -> &Any;
}

#[derive(Debug, PartialEq)]
pub struct NumBound {
    pub num: u64,
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
    pub regex: Regex,
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
