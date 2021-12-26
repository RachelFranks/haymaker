//
// Copyright 2021, Rachel Franks. All rights reserved
//

const BLUE: &'static str = "\x1b[34;1m";
const GREY: &'static str = "\x1b[90m";
const MINT: &'static str = "\x1b[38;5;48;1m";
const RED: &'static str = "\x1b[31;1m";
const RESET: &'static str = "\x1b[0;0m";
const YELLOW: &'static str = "\x1b[33;1m";

pub trait Color {
    fn color(&self, color: &str) -> String;

    fn blue(&self) -> String;
    fn clear(&self) -> String;
    fn grey(&self) -> String;
    fn mint(&self) -> String;
    fn red(&self) -> String;
    fn yellow(&self) -> String;
}

#[rustfmt::skip]
impl<T> Color for T where T: std::convert::AsRef<str> {

    fn color(&self, color: &str) -> String {
        format!("{}{}{}", color, self.as_ref(), RESET)
    }

    fn blue(&self)   -> String { self.color(BLUE)   }
    fn clear(&self)  -> String { self.color(RESET)  }
    fn grey(&self)   -> String { self.color(GREY)   }
    fn mint(&self)   -> String { self.color(MINT)   }
    fn red(&self)    -> String { self.color(RED)    }
    fn yellow(&self) -> String { self.color(YELLOW) }
}
