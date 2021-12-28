use crate::color::Color;
use itertools::Itertools;
use regex::Captures;
use regex::Regex;
use std::collections::{BTreeMap, HashSet};

pub fn derive(mut text: String, vars: &BTreeMap<String, String>) -> String {
    //

    pub fn left_derive(text: &mut String, vars: &BTreeMap<String, String>) -> bool {
        // recursively replace the leftmost instance of each @() or @"" in the string

        let mut stack: usize = 0;
        let mut start = None;

        text.push(' ');
        text.push(' ');

        for ((offset, curr), (_, next), (_, then)) in text.clone().char_indices().tuple_windows() {
            //

            if (curr, next) == ('@', '(') {
                if stack == 0 {
                    start = Some((offset, then));
                }

                stack += 1;
            }

            if curr == '@' && stack == 0 {
                let regex = Regex::new(r"^@[a-zA-Z0-9']+").unwrap();

                let replacement = regex.replacen(&text[offset..], 1, |caps: &Captures| {
                    let capture = &caps[0][1..];

                    let substitution = match vars.get(capture) {
                        Some(substitution) => substitution,
                        None => "",
                    };

                    format!("{}", substitution)
                });

                *text = text[..offset].to_owned() + &replacement;
                text.pop();
                text.pop();
                return true;
            }

            if curr == ')' {
                stack = stack.saturating_sub(1);

                if stack != 0 {
                    continue;
                }

                if let Some((at_sign, first_char)) = start {
                    start = None;

                    let inner = at_sign + 2;

                    let found = match first_char.is_alphanumeric() {
                        true => String::from("@") + &text[inner..offset],
                        false => text[inner..offset].to_owned(),
                    };

                    let replacement = subcall(derive(found, vars), false);

                    *text = match offset + 1 < text.len() {
                        true => text[..at_sign].to_owned() + &replacement + &text[(offset + 1)..],
                        false => text[..at_sign].to_owned() + &replacement,
                    };

                    text.pop();
                    text.pop();
                    return true;
                }
            }
        }

        text.pop();
        text.pop();
        false
    }

    let mut steps = vec![];

    let before = text.clone();

    while left_derive(&mut text, vars) {
        steps.push(text.clone());
    }

    println!("Derive {}", before);
    for step in steps {
        println!("  {}", step.grey());
    }

    text
}

fn subcall(mut text: String, debug: bool) -> String {
    let part_regex = Regex::new(r"(\S+)\s*(\S.*)?").unwrap();
    let args_regex = Regex::new(r#"'[^']*'|"[^"]*"|\S+"#).unwrap();
    //let args_regex = Regex::new(r#"'[^']*'|\S+"#).unwrap();

    let mut parts = text.split('|').into_iter();
    let mut state = parts.next().unwrap().trim().to_owned();

    if debug {
        println!("{} {}", "subcall".blue(), &text.trim());
        println!("  {} {}", "input".grey(), &state);
    }

    for mut part in parts {
        let mut inputs = vec![];
        let mut quoted_inputs = HashSet::new();
        let mut quoted_args = HashSet::new();

        for (index, input) in args_regex.find_iter(&state).enumerate() {
            let mut input = input.as_str();

            let has_quotes =
                input.chars().next() == Some('\'') && input.chars().rev().next() == Some('\'');

            if has_quotes && input.len() >= 2 {
                input = &input[1..(input.len() - 1)];
                quoted_inputs.insert(index);
            }
            inputs.push(input);
        }

        if debug {
            print!("  {} ", "state".grey());
            for (index, input) in inputs.iter().enumerate() {
                let wrap = match quoted_inputs.contains(&index) {
                    true => "'".grey(),
                    false => "".to_owned(),
                };
                match index {
                    0 => print!("{}{}{}", wrap, input, wrap),
                    _ => print!("{} {}{}{}", ",".grey(), wrap, input, wrap),
                }
            }
            println!();
        }

        let (command, args) = match part_regex.captures(part) {
            Some(caps) => {
                let command = caps.get(1).unwrap().as_str();

                let args_str = match caps.get(2) {
                    Some(args) => args.as_str().trim(),
                    None => "",
                };

                let mut args = vec![];

                for (index, arg) in args_regex.find_iter(args_str).enumerate() {
                    let mut arg = arg.as_str();

                    let has_quotes =
                        arg.chars().next() == Some('\'') && arg.chars().rev().next() == Some('\'');

                    if has_quotes && arg.len() >= 2 {
                        arg = &arg[1..(arg.len() - 1)];
                        quoted_args.insert(index);
                    }
                    args.push(arg);
                }

                (command, args)
            }
            None => {
                if debug {
                    println!("  {} {}", "error".grey(), "command not found".red());
                    println!();
                }
                return String::from("");
            }
        };

        if debug {
            let mut line = match args.is_empty() {
                true => format!("   {} {}", "call".grey(), &command),
                false => format!("   {} {} {}", "call".grey(), &command, "with args".grey()),
            };

            for (index, arg) in args.iter().enumerate() {
                let wrap = match quoted_args.contains(&index) {
                    true => "'".grey(),
                    false => "".to_owned(),
                };
                match index {
                    0 => line += &format!(" {}{}{}", wrap, arg, wrap),
                    _ => line += &format!("{} {}{}{}", ",".grey(), wrap, arg, wrap),
                }
            }

            println!("{}", line);
        }

        match command {
            "debug_nothing" => {
                state = inputs.iter().join(" ");
            }
            "debug_dash" => {
                state = inputs.iter().map(|s| s.replace(|_| true, "-")).join(" ");
            }
            "add" => {
                inputs.extend(args);
                state = inputs.iter().join(" ");
            }
            "append" => {
                let mut outputs = vec![];
                for input in inputs {
                    for arg in &args {
                        outputs.push(format!("{}{}", input, arg));
                    }
                }
                state = outputs.join(" ");
            }
            "compact" => {
                let mut output = vec![];
                for input in state.split(' ') {
                    if input != "" {
                        output.push(input);
                    }
                }
                state = output.join(" ");
            }
            "concat" => {
                state = inputs.iter().join("");
            }
            "exclude" => {
                state = inputs.iter().filter(|s| !args.contains(&s)).join(" ");
            }
            "include" => {
                state = inputs.iter().filter(|s| args.contains(&s)).join(" ");
            }
            "prepend" => {
                let mut outputs = vec![];
                for input in inputs {
                    for arg in &args {
                        outputs.push(format!("{}{}", arg, input));
                    }
                }
                state = outputs.join(" ");
            }
            "split" => {
                /*let arg = match args.get(0) {
                    Some(arg) => arg,
                    None => " ",
                };*/
                //state = state.split(args).join(" ");
            }
            "unquote" => {
                let mut outputs = vec![];
                for mut input in inputs {
                    let has_quotes = input.chars().next() == Some('\"')
                        && input.chars().rev().next() == Some('\"');

                    if has_quotes && input.len() >= 2 {
                        input = &input[1..(input.len() - 1)];
                    }
                    outputs.push(input);
                }
                state = outputs.join(" ");
            }
            "quote" => {
                state = inputs.iter().map(|s| format!("'{}'", s)).join(" ");
            }
            unknown => {
                if debug {
                    println!(
                        "  {} {} is not a valid command",
                        "error".grey(),
                        unknown.red()
                    );
                    println!();
                    return String::from("");
                }
            }
        }

        if debug {
            println!("    {} {}", "out".grey(), &state);
        }

        //println!("produce: {}", &state);
    }

    if debug {
        println!();
    }

    state.trim().to_owned()
}

#[test]
fn test_subcalls() {
    #[rustfmt::skip]
    let cases = [
        ("a bb ccc", "a bb ccc"),
        ("a bb ccc | invalid | add x", ""),
        ("a bb ccc | debug_nothing |", ""),
        ("", ""),
        ("|", ""),

        ("a bb ccc | concat", "abbccc"),
        ("a bb ccc | include bb", "bb"),
        ("a bb ccc | exclude bb", "a ccc"),
        ("a bb ccc | append x y", "ax ay bbx bby cccx cccy"),

        ("a bb ccc | prepend xxy ''", "xxya a xxybb bb xxyccc ccc"),
        ("a bb ccc | prepend xxy \"\"", r#"xxya ""a xxybb ""bb xxyccc ""ccc"#),

        ("a bb ccc | concat | debug_dash | add xx yyy z | debug_nothing", "------ xx yyy z"),
        ("a bb ccc \" \" | debug_nothing a b \"< >\"", "a bb ccc \" \""),
        ("a bb ccc \" \" | debug_dash", "- -- --- ---"),


        /*(r#" "ddd" " " "d d" "d  " " | unquote"#, "ddd   d d d   \""),
        (r#" '' ' ' '""' 'a"b"c' a"b" '" "b' | quote | debug_nothing"#, ""),        
        (r#"a a"b"c a"b"c"d " "b e | quote"#, r#"a"b"c"#),
        (r#"a a"b"c"d " "b e | debug_nothing"#, r#"a"b"c"#),*/
        
    ];

    for (case, correct) in cases {
        assert_eq!(&subcall(case.to_string(), true), &correct);
    }
}
