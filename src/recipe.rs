use crate::console::Color;
use crate::derive::derive;
use crate::parsed::Rule;
use std::collections::BTreeMap;
use std::process::Command;

pub struct Recipe {
    pub rule: Rule,
    pub source: Vec<String>,
}

impl From<Rule> for Recipe {
    fn from(rule: Rule) -> Self {
        Recipe {
            rule,
            source: vec![],
        }
    }
}

impl Recipe {
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

        for line in &self.source {
            println!("\t{}", line.grey());
        }
    }

    pub fn execute(&self, globals: &BTreeMap<String, String>) {
        let mut vars = globals.clone();

        let mut all = vec![];
        let mut out = vec![];

        for (index, input) in self.rule.steps.iter().flatten().enumerate() {
            vars.insert(format!("{}", index + 1), input.clone());
            all.push(input.clone());
        }
        for (index, output) in self.rule.outputs.iter().enumerate() {
            vars.insert(format!("{}'", index + 1), output.clone());
            out.push(output.clone());
        }

        vars.insert(String::from("all"), all.join(" "));
        vars.insert(String::from("out"), out.join(" "));

        for line in &self.source {
            let line = derive(line.clone(), &mut vars);

            println!("bash: {}", line.grey());
            let command = Command::new("sh").arg("-c").arg(line).output();

            let output = match command {
                Ok(output) => output,
                Err(err) => panic!("{}", err),
            };

            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
}
