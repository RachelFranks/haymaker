//
// Copyright 2021, Rachel Franks. All rights reserved
//

pub const BLUE: &'static str = "\x1b[34;1m";
pub const DIM: &'static str = "\x1b[2m";
pub const GREY: &'static str = "\x1b[0;0m\x1b[90m";
pub const MINT: &'static str = "\x1b[38;5;48;1m";
pub const PINK: &'static str = "\x1b[38;5;161;1m";
pub const RED: &'static str = "\x1b[31;1m";
pub const CLEAR: &'static str = "\x1b[0;0m";
pub const WHITE: &'static str = "\x1b[0;1m";
pub const YELLOW: &'static str = "\x1b[33;1m";

pub trait Color {
    fn color(&self, color: &str) -> String;

    fn blue(&self) -> String;
    fn dim(&self) -> String;
    fn clear(&self) -> String;
    fn grey(&self) -> String;
    fn mint(&self) -> String;
    fn pink(&self) -> String;
    fn red(&self) -> String;
    fn white(&self) -> String;
    fn yellow(&self) -> String;
}

#[rustfmt::skip]
impl<T> Color for T where T: std::fmt::Display {

    fn color(&self, color: &str) -> String {
        format!("{}{}{}", color, self, CLEAR)
    }

    fn blue(&self)   -> String { self.color(BLUE)   }
    fn dim(&self)    -> String { self.color(DIM)    }
    fn clear(&self)  -> String { self.color(CLEAR)  }
    fn grey(&self)   -> String { self.color(GREY)   }
    fn mint(&self)   -> String { self.color(MINT)   }
    fn pink(&self)   -> String { self.color(PINK)   }
    fn red(&self)    -> String { self.color(RED)    }
    fn white(&self)  -> String { self.color(WHITE)  }
    fn yellow(&self) -> String { self.color(YELLOW) }
}

pub fn print_source_error(
    kind: &str,
    message: &str,
    filename: &str,
    line: &str,
    num: usize,
    column: usize,
) {
    let len = num.to_string().len();
    let margin = " ".repeat(len);
    let pipe = "║".blue();

    let arrow = "╔═══════".blue();
    let position = format!("line {} column {}", num.blue(), column.blue());
    let editor = format!("({}:{}:{})", filename, num, column).grey();

    let line = line.replace('\t', " ");

    println!();
    println!("{}: {}", kind.red(), message);
    println!("{} {} {} {} {}", margin, arrow, filename.blue(), position, editor);
    println!("{} {}", margin, pipe);
    println!("{} {} {}", num.blue(), pipe, line);
    println!("{} {} {}{}", margin, pipe, " ".repeat(column), "^".red());
    println!();
}

pub fn print_processed_error(
    kind: &str,
    message: &str,
    filename: &str,
    line: &str,
    info: Vec<String>,
    num: usize,
    column: usize,
) {
    let len = num.to_string().len();
    let margin = " ".repeat(len);
    let pipe = "║".blue();

    let arrow = "╔═══════".blue();
    let position = format!("line {}", num.blue());
    let editor = format!("({}:{})", filename, num).grey();

    let line = line.replace('\t', " ");

    println!();
    println!("{}: {}", kind.red(), message);
    println!("{} {} {} {} {}", margin, arrow, filename.blue(), position, editor);
    println!("{} {}", margin, pipe);
    println!("{} {} {}", num.blue(), pipe, line);
    println!("{} {} {}{}", margin, pipe, " ".repeat(column), "^".red());
    println!("{} {}", margin, pipe);

    for line in info {
        let line = line.replace('\t', " ");
        println!("{} {} {}", margin, pipe, line);
    }
    println!();
}
