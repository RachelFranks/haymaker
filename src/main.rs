//
// Copyright 2021, Rachel Franks. All rights reserved
//

use itertools::Itertools;
use petgraph::{stable_graph::StableGraph, Direction};
use regex::Regex;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::color::Color;
use crate::parsed::MakeLine;
use crate::recipe::Recipe;

use lalrpop_util::lalrpop_mod;
//use lalrpop_util::ParseError;
use crate::def::DefParser;
lalrpop_mod!(def);

mod color;
mod derive;
mod parsed;
mod recipe;
mod text;

#[derive(Debug, StructOpt)]
#[structopt(name = "paramake", about = "A fearlessly parallel build system")]
struct Opt {
    #[structopt(parse(from_os_str))]
    makefile: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let lines = match File::open(&opt.makefile) {
        Ok(file) => BufReader::new(file).lines(),
        Err(err) => panic!("Unable to open file {}: {}", opt.makefile.display(), err),
    };

    let vars_regex = Regex::new(r"[a-zA-Z0-9]+").unwrap();
    let blank_regex = Regex::new(r"^\s*$").unwrap();

    let mut recipes: Vec<Recipe> = vec![];
    let mut variables = BTreeMap::new();

    for line in lines.filter_map(|x| x.ok()) {
        // Makefiles are context-sensitive, so we must determine how to handle each line

        if blank_regex.is_match(&line) {
            // skip blanks for performance
            continue;
        }

        if line.starts_with("\t") {
            // shell source can have arbitrary text & starts after the tab

            let recipe = match recipes.last_mut() {
                Some(recipe) => recipe,
                None => {
                    panic!("No recipe");
                }
            };

            let source_line = line[1..].to_string();
            recipe.source.push(source_line);
            continue;
        }

        if line.contains("=") {
            // variable assignments

            let mut sides: Vec<_> = line.split('=').collect();
            sides.reverse();

            for (value, dest) in sides.into_iter().tuple_windows() {
                let value = value.trim();
                let assigns = vars_regex.captures_iter(dest).map(|x| x[0].to_string());

                for assign in assigns {
                    variables.insert(assign, value.to_string());
                }
            }

            continue;
        }

        let parsed = match DefParser::new().parse(&line) {
            Ok(Some(parsed)) => parsed,
            Err(err) => panic!("error parsing\n{}\n{}", line.red(), err),
            Ok(_) => continue,
        };

        if let MakeLine::Rule(rule) = parsed {
            let recipe = Recipe::from(rule);
            recipes.push(recipe);
            continue;
        }

        if let MakeLine::Import(_import) = parsed {
            continue;
        }
    }

    for recipe in &recipes {
        recipe.print();
        println!();
    }

    let mut graph: StableGraph<Recipe, ()> = StableGraph::new();
    //let mut nodes: BTreeMap::new();

    for recipe in recipes {
        let _node = graph.add_node(recipe);
    }

    while graph.node_count() > 0 {
        let ready: Vec<_> = graph.externals(Direction::Outgoing).collect();

        for node in ready {
            let recipe = &graph[node];
            recipe.execute(&variables);
            graph.remove_node(node);
        }
    }

    /*println!("Variables");
    for (variable, value) in variables {
        println!("  {}: {}", variable, value);
    }*/
}
