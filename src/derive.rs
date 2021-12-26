use crate::color::Color;
use itertools::Itertools;
use regex::Captures;
use regex::Regex;
use std::collections::BTreeMap;

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

                    format!("<{}>", substitution)
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

                    let replacement = derive(found, vars);

                    *text = match offset + 1 < text.len() {
                        true => {
                            text[..at_sign].to_owned()
                                + "<"
                                + &replacement
                                + ">"
                                + &text[(offset + 1)..]
                        }
                        false => text[..at_sign].to_owned() + "<" + &replacement + ">",
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
