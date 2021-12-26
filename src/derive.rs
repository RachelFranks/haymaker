
use crate::color::Color;
use regex::Regex;
use regex::Captures;
use std::collections::BTreeMap;

pub fn derive(mut text: String, vars: &BTreeMap<String, String>) -> String {
    let regex = Regex::new(r"@[a-zA-Z0-9']+").unwrap();

    while let Some(_) = regex.find(&text) {

        let new = regex.replace(&text, |caps: &Captures| {

            let capture = &caps[0][1..];

            let substitution = match vars.get(capture) {
                Some(substitution) => substitution,
                None => "",
            };
            
            format!("{}", substitution)
        }).to_string();

        println!("{} {} {}", text.blue(), "=>", new.blue());
        
        text = new;
    }

    text
}
