use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub awww: Awww,
}

impl easy_storage::Storeable for Config {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Awww {
    pub args: Vec<String>,
}

impl Default for Awww {
    fn default() -> Self {
        let f = [
            "--transition-type",
            "center",
            "--transition-duration",
            "0.5",
        ]
        .map(String::from);
        Self { args: f.to_vec() }
    }
}
