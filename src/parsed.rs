pub enum MakeLine {
    Rule(Rule),
}

pub struct Rule {
    pub outputs: Vec<String>,
    pub steps: Vec<Vec<String>>,
}
