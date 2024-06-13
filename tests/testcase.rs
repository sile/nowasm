use orfail::OrFail;
use serde::Deserialize;
use std::path::{Path, PathBuf};

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
    Module {
        filename: PathBuf,
    },
    AssertReturn {
        action: Action,
        expected: Vec<Value>,
    },
    AssertTrap {
        action: Action,
        text: String,
        expected: Vec<Value>,
    },
    AssertMalformed {
        filename: PathBuf,
        text: String,
        module_type: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Invoke { field: String, args: Vec<Value> },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Value {
    I32 {
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    I64 {
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    F32 {
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    F64 {
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
}
