use std::path::PathBuf;

pub enum MakeLine {
    Rule(Rule),
    Import(Import),
}

pub struct Rule {
    pub outputs: Vec<String>,
    pub steps: Vec<Vec<String>>,
}

pub struct Import {
    pub files: Vec<PathBuf>,
}
