//
// Copyright 2021, Rachel Franks. All rights reserved
//

use crate::color::Color;
use crate::comments::uncomment;
use crate::parsed::MakeLine;
use crate::recipe::Recipe;

use itertools::Itertools;
use petgraph::{stable_graph::StableGraph, Direction};
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

use lalrpop_util::lalrpop_mod;
//use lalrpop_util::ParseError;
use crate::def::DefParser;
lalrpop_mod!(def);

mod color;
mod comments;
mod derive;
mod parsed;
mod recipe;
mod regexes;
mod text;

#[derive(Debug, StructOpt)]
#[structopt(name = "haymaker", about = "A fearlessly parallel build system")]
struct Opt {
    #[structopt(parse(from_os_str))]
    hayfile: Option<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();

    let hayfile = match opt.hayfile {
        Some(hayfile) => hayfile,
        None => {
            let defaults = ["hayfile", "Hayfile", "makefile", "Makefile"];

            match defaults.into_iter().find(|file| Path::new(file).exists()) {
                Some(hayfile) => Path::new(hayfile).to_path_buf(),
                None => {
                    println!("No {} in current directory", "hayfile".red());
                    std::process::exit(1);
                }
            }
        }
    };

    let haysource = match std::fs::read_to_string(&hayfile) {
        Ok(haysource) => haysource,
        Err(err) => {
            println!("Could not open {}\n{}", hayfile.to_string_lossy().red(), err);
            std::process::exit(1);
        }
    };

    let mut recipes: Vec<Recipe> = vec![];
    let mut variables = BTreeMap::new();

    for line in uncomment(&haysource, "") {
        // Hayfiles are context-sensitive, so we must determine how to handle each line

        if line.trim() == "" {
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
                let assigns = regexes::VAR.captures_iter(dest).map(|x| x[0].to_string());

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
