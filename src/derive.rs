//

use crate::color::Color;
use itertools::Itertools;
use regex::Captures;
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn derive(mut text: String, vars: &mut BTreeMap<String, String>) -> String {
    pub fn left_derive(text: &mut String, vars: &mut BTreeMap<String, String>) -> bool {
        // recursively replace the leftmost instance of each @() or @"" in the string

        let mut stack: usize = 0;
        let mut start = None;
        let mut quoted = false;

        text.push(' ');
        text.push(' ');

        for ((offset, curr), (_, next), (_, then)) in text.clone().char_indices().tuple_windows() {
            //

            if curr == '\'' {
                quoted = !quoted;
            }

            if quoted {
                continue;
            }

            if (curr, next) == ('@', '(') {
                if stack == 0 {
                    start = Some((offset, then));
                }

                stack += 1;
            }

            if curr == '@' && stack == 0 {
                let regex = Regex::new(r"^@[a-zA-Z0-9_]+").unwrap();

                if !regex.is_match(&text[offset..]) {
                    continue;
                }

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
                    let inner = at_sign + 2;

                    let found = match first_char.is_alphanumeric() {
                        true => String::from("@") + &text[inner..offset],
                        false => text[inner..offset].to_owned(),
                    };

                    let (replacement, defs) = subcall(derive(found, vars), true);
                    vars.extend(defs.into_iter());

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

    let var_regex = Regex::new(r"(@[a-zA-Z0-9_]+)").unwrap();
    let pretty = text.trim().replace("|", &"|".blue());
    let pretty = var_regex.replace_all(&pretty, "$1".pink());

    let var_regex = Regex::new(r"@\(([a-zA-Z0-9_]+)").unwrap();
    let pretty = var_regex.replace_all(&pretty, format!("@({}", "$1".pink()));

    let mut steps = vec![];

    let mut times = 0;

    while left_derive(&mut text, vars) {
        steps.push(text.clone());
        times += 1;

        if times > 16 {
            println!("{} {}", "process".pink(), pretty);
            for step in steps {
                println!("        {}", step.grey());
            }
            panic!("too many times");
        }
    }

    println!("{} {}", "process".pink(), pretty);
    //println!("{} {}", "process".pink(), before.trim().replace("@", &"@".blue()).replace("|", &"|".blue()));
    for step in steps {
        println!("        {}", step.grey());
    }

    text
}

/*fn split_balanced<'a, 'b>(text: &'a str, on: &char, on: &'b str) -> &'a str {

}*/

fn subcall(text: String, debug: bool) -> (String, HashMap<String, String>) {
    let mut defs = HashMap::new();

    let part_regex = Regex::new(r"(\S+)\s*(\S.*)?").unwrap();
    let args_regex = Regex::new(r#"'[^']*'|"[^"]*"|\S+"#).unwrap();

    let mut parts = vec![];
    let mut start = 0;
    let mut quoted = false;
    for (offset, c) in text.char_indices() {
        if c == '\'' {
            quoted = !quoted;
        }
        if !quoted && c == '|' {
            parts.push(&text[start..offset]);
            start = offset + 1;
        }
    }
    parts.push(&text[start..]);

    let mut parts = parts.into_iter();
    let mut state = parts.next().unwrap().trim().to_owned();

    if debug {
        let full = text.trim().replace("|", &"|".blue());
        println!("{} {}{}{}", "\nsubcall".blue(), "@(".blue(), full, ")".blue());
        println!("  {} {}", "input".grey(), &state);
    }

    macro_rules! error {
        ($format:expr $(,$args:expr)* $(,)?) => {{
            let message = format!($format, $($args,)*);
            if debug {
                println!("  {} {}", "error".grey(), message.red());
            }
            return (String::new(), HashMap::new());
        }};
    }

    macro_rules! number {
        ($expr:expr, $onerr:expr) => {{
            let value: usize = match $expr.parse() {
                Ok(value) => value,
                Err(_) => error!("{}: {} is not a number", $onerr, $expr),
            };
            value
        }};
    }

    for part in parts {
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
            None => error!("command not found"),
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
            "count" => {
                state = inputs.len().to_string();
            }
            "concat" => {
                state = inputs.iter().join("");
            }
            "noop" => {
                state = inputs.iter().join(" ");
            }
            "debug_dash" => {
                state = inputs.iter().map(|s| s.replace(|_| true, "-")).join(" ");
            }
            "include" => {
                state = inputs.iter().filter(|s| args.contains(&s)).join(" ");
            }
            "exclude" => {
                state = inputs.iter().filter(|s| !args.contains(&s)).join(" ");
            }
            "quote" => {
                state = inputs.iter().map(|s| format!("'{}'", s)).join(" ");
            }
            "add" => {
                inputs.extend(args);
                state = inputs.iter().join(" ");
            }
            "sort" => {
                inputs.sort();
                state = inputs.iter().join(" ");
            }
            "first" => match inputs.get(0) {
                Some(first) => state = first.to_string(),
                None => error!("no first input"),
            },
            "last" => match inputs.last() {
                Some(first) => state = first.to_string(),
                None => error!("no last input"),
            },
            "def" => {
                for arg in &args {
                    defs.insert(arg.to_string(), state.to_owned());
                }
            }
            "drop" => {
                let count = match args.get(0) {
                    Some(arg) => number!(arg, "drop"),
                    None => 1,
                };
                state = inputs.iter().skip(count).join(" ");
            }
            "pop" => {
                let count = match args.get(0) {
                    Some(arg) => number!(arg, "pop"),
                    None => 1,
                };
                state = inputs.iter().take(inputs.len().saturating_sub(count)).join(" ");
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
            "prepend" => {
                let mut outputs = vec![];
                for input in inputs {
                    for arg in &args {
                        outputs.push(format!("{}{}", arg, input));
                    }
                }
                state = outputs.join(" ");
            }
            "between" => {
                let first = match args.get(0) {
                    Some(arg) => number!(arg, "between").saturating_sub(1),
                    None => 0,
                };
                let second = match args.get(1) {
                    Some(arg) => number!(arg, "between"),
                    None => inputs.len(),
                };
                state = inputs.iter().skip(first).take(second.saturating_sub(first)).join(" ");
            }
            "has" => {
                let mut found = false;
                'outer: for input in inputs {
                    for arg in &args {
                        if &input == arg {
                            // will change to a regex, hence the double loop
                            found = true;
                            break 'outer;
                        }
                    }
                }
                if !found {
                    state = String::new();
                }
            }
            "index" => {
                let mut outputs = vec![];
                for mut arg in args {
                    let mut flip = false;

                    if arg.chars().next() == Some('-') {
                        arg = &arg[1..];
                        flip = true;
                    }

                    let offset = number!(arg, "index");

                    let index = match flip {
                        true => match inputs.len() >= offset {
                            true => inputs.len() - offset,
                            false => continue,
                        },
                        false => offset.saturating_sub(1),
                    };

                    if let Some(input) = inputs.get(index) {
                        outputs.push(input);
                    }
                }
                state = outputs.into_iter().join(" ");
            }

            "filter" => {}
            "sift" => {}

            "replace" => {}

            "shell" => {
                let mut child = Command::new("sh")
                    .arg("-c")
                    .arg(args.iter().join(" "))
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();

                child
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(inputs.into_iter().join(" ").as_bytes())
                    .unwrap();

                let output = child.wait_with_output().unwrap();

                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    println!("  {} {}: {}", "error".grey(), "shell failure".red(), error);
                    return (String::new(), HashMap::new());
                }

                state = String::from_utf8_lossy(&output.stdout).to_string();
            }
            "split" => {
                let mut outputs: Vec<_> = inputs;
                for arg in &args {
                    outputs = outputs
                        .into_iter()
                        .map(|s| s.split(arg))
                        .flatten()
                        .map(|s| s)
                        .collect();
                }
                state = outputs.into_iter().filter(|s| s != &"").join(" ");
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
            "stop" => {
                println!("    {} {}", "out".grey(), &state);
                return (state, defs);
            }
            "error" => error!("called error"),
            "suppress_errors" => {}
            unknown => {
                if debug {
                    println!("  {} {} is not a valid command", "error".grey(), unknown.red());
                    return (String::from(""), HashMap::new());
                }
            }
        }

        if debug {
            println!("    {} {}", "out".grey(), &state);
        }
    }

    if debug {
        for (def, value) in &defs {
            println!("    {} {} {} {}", "def".pink(), def, "≡".pink(), value);
        }
        println!();
    }

    (state.trim().to_owned(), defs)
}

#[test]
fn test_subcalls() {
    #[rustfmt::skip]
    let cases = [
        ("a bb ccc", "a bb ccc"),
        ("a bb ccc | invalid | add x", ""),
        ("a bb ccc | noop |", ""),
        ("", ""),
        ("|", ""),

        ("a bb ccc | concat", "abbccc"),
        ("a bb ccc | include bb", "bb"),
        ("a bb ccc | exclude bb", "a ccc"),
        ("a bb ccc | append x y", "ax ay bbx bby cccx cccy"),

        ("a bb ccc | prepend xxy ''", "xxya a xxybb bb xxyccc ccc"),
        ("a bb ccc | prepend xxy \"\"", r#"xxya ""a xxybb ""bb xxyccc ""ccc"#),

        ("a bb ccc | concat | debug_dash | add xx yyy z | noop", "------ xx yyy z"),
        ("a bb ccc \" \" | noop a b \"< >\"", "a bb ccc \" \""),
        ("a bb ccc \" \" | debug_dash", "- -- --- ---"),
        ("a bb ccc ' ' | debug_dash", "- -- --- -"),

        ("wow,this,is,cool | split ,", "wow this is cool"),
        ("wow,this,is,cool | split , o", "w w this is c l"),
        ("wow,this,is,cool | split is t s", "wow, h , ,cool"),

        ("a definition | def key1 key2", "a definition"),

        ("please remove my es | append ~ | split e | concat | split ~", "plas rmov my s"),
        ("a b c d | pop | pop", "a b"),
        ("a b c d | pop | drop", "b c"),
        ("a b c d | pop 2 | drop", "b"),
        ("a b c d | drop 2 | pop", "c"),

        ("one two three four 5 6 seven | count", "7"),
        ("one two three four 5 6 seven | between 3 5", "three four 5"),
        ("one two three four 5 6 seven | between 6", "6 seven"),
        ("one two three four 5 6 seven | index 4 1 5", "four one 5"),
        ("one two three four 5 6 seven | index 8 -2 3 -1 -8 -7 0", "6 three seven one one"),
        ("one two three four 5 6 seven | first", "one"),
        ("one two three four 5 6 seven | last", "seven"),
        ("one two three four 5 6 seven | index 8 lol | add xx", ""),
        (" | first | add xx", ""),
        (" | last  | add xx", ""),

        ("0 3 9 10 33 | sort", "0 10 3 33 9"),
        ("0 3 9 10 33 | has yy 10", "0 3 9 10 33"),

        ("this | stop  | add xx", "this"),
        ("this | error | add xx", ""),

        (" | shell echo -n hello", "hello"),
        (" | shell fail -n hello", ""),
        ("humanity <3 | shell wc -c", "11"),
        
        ("a a bb bb a c | shell cat '|' wc -c", "13"),
        
        // TODO: decide on consistent quoting rules
        /*(r#" "ddd" " " "d d" "d  " " | unquote"#, "ddd   d d d   \""),
        (r#" '' ' ' '""' 'a"b"c' a"b" '" "b' | quote | noop"#, ""),        
        (r#"a a"b"c a"b"c"d " "b e | quote"#, r#"a"b"c"#),
        (r#"a a"b"c"d " "b e | noop"#, r#"a"b"c"#),*/
    ];

    let mut defs = HashMap::new();

    for (case, correct) in cases {
        let (text, new_defs) = subcall(case.to_string(), true);
        defs.extend(new_defs.into_iter());
        assert_eq!(&text, &correct);
    }

    assert_eq!(defs.get("key1"), Some(&String::from("a definition")));
    assert_eq!(defs.get("key2"), Some(&String::from("a definition")));
}

#[test]
fn test_derivation() {
    let mut vars = BTreeMap::new();
    vars.insert(String::from("out"), String::from("bin"));
    vars.insert(String::from("1"), String::from("aa"));
    vars.insert(String::from("2"), String::from("bb"));

    #[rustfmt::skip]
    let cases = [
        ("@out '@out' @( '@' | noop)2", "bin '@out' bb"),
        ("@1 '@2' @('@' | noop)", "aa '@2' @"),
    ];

    for (case, correct) in cases {
        let line = derive(case.to_string(), &mut vars);
        assert_eq!(&line, &correct);
        println!();
    }
}
