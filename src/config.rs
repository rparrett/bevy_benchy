use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub commits: Vec<Commit>,
    pub benches: Vec<Bench>,
    pub frames: u32,
}

#[derive(Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct Bench {
    pub example: String,
    #[serde(default)]
    pub example_args: Vec<String>,
    pub label: Option<String>,
}
impl Bench {
    pub fn label(&self) -> String {
        self.label
            .clone()
            .unwrap_or_else(|| format!("{} {}", self.example, self.example_args.join(" ")))
            .to_string()
    }
}

#[derive(Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct Commit {
    pub commit: String,
    pub label: Option<String>,
}
impl Commit {
    pub fn label(&self) -> &str {
        self.label.as_ref().unwrap_or(&self.commit)
    }
}
