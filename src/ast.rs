

pub enum ShellNode {
    Root(Vec<ShellNode>),
    Subcall(Vec<ShellNode>, bool),
    Expand(String),
    Text(String),
}
