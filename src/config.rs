use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub commits: Vec<Commit>,
    pub benches: Vec<Bench>,
}

#[derive(Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct Bench {
    pub example: String,
    pub example_args: Vec<String>,
    pub label: String,
}

#[derive(Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct Commit {
    pub commit: String,
    pub label: String,
}
