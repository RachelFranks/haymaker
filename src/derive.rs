//

use crate::console::Color;
use crate::regexes;
use crate::text::Text;

use itertools::Itertools;
use regex::Captures;
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write;
use std::process::{Command, Stdio};

type VarMap = BTreeMap<String, String>;

pub fn derive(text: &str, vars: &mut VarMap, debug: bool) -> String {
    //

    let mut text = text.to_owned();
    let mut intos = vec![];
    let mut subcalls = vec![];
    let mut steps = match debug {
        true => vec![text.clone()],
        false => vec![],
    };

    'top: loop {
        let padding = vec![(text.len(), ' '), (text.len() + 1, ' ')];
        let mut iter = text.char_indices().chain(padding).tuple_windows();

        let mut start = None;
        let mut end = None;
        let mut quoted = false;

        for ((offset, c), (_, n), (_, a)) in iter {
            if c == '\'' {
                quoted = !quoted;
            }
            if quoted {
                continue;
            }

            if let ('@', '(') = (c, n) {
                start = Some(offset);
            }

            if start.is_some() && c == ')' {
                end = Some(offset + 1);
                break;
            }

            if c == '@' {
                if let Some(mat) = regexes::VAR_AT_WITH_SIGN.find(&text[offset..]) {
                    let start = offset + mat.start();
                    let end = offset + mat.end();
                    let var = &mat.as_str()[1..];

                    let replace = match vars.get(var) {
                        Some(replace) => replace,
                        None => "",
                    };

                    if debug {
                        intos.push(format!("{} » {}", var, replace.or_quotes()));
                    }

                    text = match end < text.len() {
                        true => text[0..start].to_owned() + &replace + &text[end..],
                        false => text[0..start].to_owned() + &replace,
                    };
                    if debug {
                        steps.push(text.clone());
                    }
                    continue 'top;
                }
            }
        }

        if let (Some(start), Some(end)) = (start, end) {
            let mut inner = text[start + 2..end - 1].to_owned();

            if let Some(mat) = regexes::VAR_AT.find(&inner) {
                let var = match vars.get(mat.as_str()) {
                    Some(replace) => replace,
                    None => "",
                };

                match mat.end() < inner.len() {
                    true => inner = var.to_owned() + &inner[mat.end()..],
                    false => inner = var.to_owned(),
                }
            }

            let (replace, printable) = subcall(&inner, vars, debug);

            text = match end < text.len() {
                true => text[0..start].to_owned() + &replace + &text[end..],
                false => text[0..start].to_owned() + &replace,
            };
            if debug {
                steps.push(text.clone());
                intos.push(format!("@(..) » {}", replace.or_quotes()));
                subcalls.push(printable)
            }
            continue;
        }

        break;
    }

    if debug {
        let mut steps = steps.into_iter();
        println!("{} {}", "derive".pink(), add_highlights(steps.next().unwrap()));

        let width = intos.iter().map(|x| x.len()).max().unwrap_or(0);

        for (step, into) in steps.zip(intos.iter()) {
            let spacing = " ".repeat(width - into.len());
            println!("  {} {} {}", into.dim(), spacing, add_highlights(step));
        }
        println!();

        for subcall in subcalls {
            println!("{}", subcall);
        }
    }

    text.to_string()
}

fn subcall(text: &str, vars: &mut VarMap, debug: bool) -> (String, String) {
    let mut printable = String::new();
    let mut defs = VarMap::new();

    let part_regex = Regex::new(r"(\S+)\s*(\S.*)?").unwrap();
    let args_regex = Regex::new(r#"'[^']*'|"[^"]*"|\S+"#).unwrap();

    let mut parts = text.split_when_balanced('|', '\'').into_iter();

    let mut state = match parts.next() {
        Some(state) => state.trim().to_owned(),
        None => String::new(),
    };

    macro_rules! save {
        () => {{
            printable += "\n";
        }};
        ($format:expr $(,$args:expr)* $(,)?) => {{
            printable += &format!($format, $($args,)*);
            printable += "\n";
        }};
        (@$format:expr $(,$args:expr)* $(,)?) => {{
            printable += &format!($format, $($args,)*);
        }};
    };

    if debug {
        let full = text.trim().replace("|", &"|".blue());
        save!("{} {}{}{}", "subcall".blue(), "@(".blue(), full, ")".blue());
        save!("  {} {}", "input".grey(), &state);
    }

    macro_rules! error {
        ($format:expr $(,$args:expr)* $(,)?) => {{
            let message = format!($format, $($args,)*);
            if debug {
                save!("  {} {}", "error".grey(), message.red());
            }
            return (String::new(), printable);
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
            save!(@"  {} ", "state".grey());
            for (index, input) in inputs.iter().enumerate() {
                let wrap = match quoted_inputs.contains(&index) {
                    true => "'".grey(),
                    false => "".to_owned(),
                };
                match index {
                    0 => save!(@"{}{}{}", wrap, input, wrap),
                    _ => save!(@"{} {}{}{}", ",".grey(), wrap, input, wrap),
                }
            }
            save!();
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

            save!("{}", line);
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
                    vars.insert(arg.to_string(), state.to_owned());
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
                    save!("  {} {}: {}", "error".grey(), "shell failure".red(), error);
                    return (String::new(), printable);
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
                save!("    {} {}", "out".grey(), &state);
                return (state, printable);
            }
            "error" => error!("called error"),
            "suppress_errors" => {}
            unknown => {
                if debug {
                    save!("  {} {} is not a valid command", "error".grey(), unknown.red());
                    return (String::from(""), printable);
                }
            }
        }

        if debug {
            save!("    {} {}", "out".grey(), &state);
        }
    }

    if debug {
        for (def, value) in &defs {
            save!("    {} {} {} {}", "def".pink(), def, "≡".pink(), value);
        }
    }

    (state.trim().to_owned(), printable)
}

fn add_highlights(text: String) -> String {
    let padding = vec![(text.len(), ' ')];
    let mut iter = text.char_indices().chain(padding).tuple_windows();
    let mut out = String::with_capacity(text.len());
    out += crate::console::GREY;

    let mut stack = 0_usize;
    let mut quoted = false;
    let mut coloring = false;

    let blue = crate::console::BLUE;
    let pink = crate::console::PINK;
    let grey = crate::console::GREY;

    while let Some(((offset, c), (_, n))) = iter.next() {
        if coloring && !c.is_alphanumeric() {
            out += grey;
            coloring = false;
        }

        if c == '\'' {
            quoted = !quoted;
        }
        if quoted {
            out.push(c);
            continue;
        }

        if let ('@', '(') = (c, n) {
            out += &format!("{}{}{}", blue, "@(", pink);
            stack += 1;
            coloring = true;
            iter.next();
            continue;
        }

        if stack > 0 && c == ')' {
            stack -= 1;
            out += &format!("{}{}{}", blue, ")", grey);
            continue;
        }

        if stack > 0 && c == '|' {
            out += &format!("{}{}{}", blue, "|", grey);
            continue;
        }

        if c == '@' && n.is_alphanumeric() {
            out += &format!("{}{}", pink, "@");
            coloring = true;
            continue;
        }

        out.push(c);
    }

    out.grey()
}

#[test]
fn test_subcalls() {
    #[rustfmt::skip]
    let cases = [
        ("a bb ccc", "a bb ccc"),
        ("a bb ccc | invalid | add x", ""),
        ("a bb ccc | noop |", "a bb ccc"),
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

    let mut vars = VarMap::new();

    for (case, correct) in cases {
        let (text, printable) = subcall(&case, &mut vars, true);
        println!("{}", printable);
        assert_eq!(&text, &correct);
    }

    assert_eq!(vars.get("key1"), Some(&String::from("a definition")));
    assert_eq!(vars.get("key2"), Some(&String::from("a definition")));
}

#[test]
fn test_derivation() {
    let mut vars = BTreeMap::new();
    vars.insert(String::from("out"), String::from("bin"));
    vars.insert(String::from("1"), String::from("aa"));
    vars.insert(String::from("2"), String::from("bb"));

    #[rustfmt::skip]
    let cases = [
        ("echo hi", "echo hi"),
        ("@1 @out @1 '@out' @( '@' | noop)2", "aa bin aa '@out' bb"),
        ("@1 '@2' @('@' | noop)", "aa '@2' @"),
        ("@(out) @out @(@(out)) @(@(out | noop)) out", "bin bin   out"),
        ("@( out) @( out | noop) @(1)", "out out aa"),
    ];

    for (case, correct) in cases {
        let line = derive(&case, &mut vars, true);
        assert_eq!(&line, &correct);
    }
}
