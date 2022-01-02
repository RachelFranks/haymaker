use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref VAR: Regex = Regex::new(r"[a-zA-Z0-9_-]+").unwrap();
}
