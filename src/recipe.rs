use crate::console::Color;
use crate::derive::{add_derivation_highlights, derive, VarMap};

use std::process::Command;

pub struct Recipe {
    pub rule: Rule,
    pub commands: Vec<ShellCommand>,
}

pub struct Rule {
    pub outputs: Vec<String>,
    pub steps: Vec<Vec<String>>,
}

pub struct ShellCommand {
    pub line: String,
    pub debug: bool,
}

impl From<Rule> for Recipe {
    fn from(rule: Rule) -> Self {
        Recipe {
            rule,
            commands: vec![],
        }
    }
}

impl Recipe {
    pub fn add_command(&mut self, line: String, debug: bool) {
        self.commands.push(ShellCommand { line, debug });
    }

    pub fn print(&self) {
        for (index, output) in self.rule.outputs.iter().enumerate() {
            let spacer = match index {
                0 => "",
                _ => " ",
            };
            print!("{}{}", spacer, output.blue());
        }
        print!("{}", ":".blue());

        for (index, step) in self.rule.steps.iter().enumerate() {
            let spacer = match index {
                0 => " ".clear(),
                _ => "| ".blue(),
            };
            print!("{}", spacer);

            for need in step {
                print!("{} ", need);
            }
        }
        println!("");

        for command in &self.commands {
            let line = add_derivation_highlights(&command.line);
            println!("\t{}", line);
        }
    }

    pub fn execute(&self, globals: &VarMap) {
        let mut vars = globals.clone();

        let mut all = vec![];
        let mut out = vec![];

        for (index, input) in self.rule.steps.iter().flatten().enumerate() {
            vars.insert(format!("{}", index + 1), input.clone());
            all.push(input.clone());
        }
        for (index, output) in self.rule.outputs.iter().enumerate() {
            vars.insert(format!("out{}", index + 1), output.clone());
            out.push(output.clone());
        }

        vars.insert(String::from("all"), all.join(" "));
        vars.insert(String::from("out"), out.join(" "));

        for command in &self.commands {
            let line = &command.line;
            let debug = command.debug;

            let line = derive(&line, &mut vars, debug);

            println!("{}", line.grey());
            let command = Command::new("sh").arg("-c").arg(line).output();

            let output = match command {
                Ok(output) => output,
                Err(err) => panic!("{}", err),
            };

            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
}
