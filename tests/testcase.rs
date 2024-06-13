use orfail::OrFail;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Testcase {
    pub commands: Vec<Command>,
}

impl Testcase {
    pub fn load<P: AsRef<Path>>(name: P) -> orfail::Result<Self> {
        let path = Path::new(file!())
            .parent()
            .or_fail()?
            .parent()
            .or_fail()?
            .join("testdata/")
            .join(&name);
        let json = std::fs::read_to_string(&path).or_fail()?;
        serde_json::from_str(&json).or_fail()
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    Module,
    AssertReturn,
    AssertTrap,
    AssertMalformed,
}
