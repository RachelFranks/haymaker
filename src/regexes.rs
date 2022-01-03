use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref VAR: Regex = Regex::new(r"[\p{Alphabetic}\pN_-]+").unwrap();
    pub static ref VAR_CHAR: Regex = Regex::new(r"^[\p{Alphabetic}\pN_-]$").unwrap();
    pub static ref VAR_AT: Regex = Regex::new(r"^@[\p{Alphabetic}\p{Number}_-]+").unwrap();
}
