use reqwest;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

const RUN_URL: &str = "https://play.rust-lang.org/execute";
const GIST_GEN_URL: &str = "https://play.rust-lang.org/meta/gist/";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunReq {
    backtrace: bool,
    channel: String,
    code: String,
    crate_type: String,
    edition: String,
    mode: String,
    tests: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct RunRes {
    success: bool,
    stdout: String,
    stderr: String,
}

#[derive(Deserialize)]
struct GistRes {
    id: String,
}

pub struct Code {
    /// Code you want to share/run.
    pub source_code: String,
    /// Rustc version, `stable`, `beta`, or `nightly`.
    pub version: String,
    /// Opt level, `debug` or `release`.
    pub mode: String,
    /// Edition, `2015` or `2018` or `2021`.
    pub edition: String,
}

impl Default for RunReq {
    fn default() -> Self {
        Self {
            backtrace: false,
            channel: "stable".to_owned(),
            code: "".to_owned(),
            crate_type: "bin".to_owned(),
            edition: "2018".to_owned(),
            mode: "debug".to_owned(),
            tests: false,
        }
    }
}

impl From<Code> for RunReq {
    fn from(code: Code) -> Self {
        Self {
            code: code.source_code,
            channel: code.version,
            mode: code.mode,
            edition: code.edition,
            ..Self::default()
        }
    }
}

impl Code {
    pub fn is_valid(&self) -> Result<bool, String> {
        if !["stable", "beta", "nightly"].contains(&self.version.to_ascii_lowercase().as_str()) {
            return Err(format!(
                "Invalid version ✖️: '{}', valid is 'stable', 'beta', 'nightly'",
                self.version
            ));
        }
        if !["debug", "release"].contains(&self.mode.to_ascii_lowercase().as_str()) {
            return Err(format!(
                "Invalid mode ✖️: '{}', valid is 'debug', 'release'",
                self.mode
            ));
        }
        if !["2015", "2018", "2021"].contains(&self.edition.as_str()) {
            return Err(format!(
                "Invalid edition ✖️: '{}', valid is '2015', '2018', '2021'",
                self.edition
            ));
        }

        Ok(true)
    }
}

impl Default for Code {
    fn default() -> Self {
        Self {
            source_code: "".to_owned(),
            version: "stable".to_owned(),
            mode: "debug".to_owned(),
            edition: "2018".to_owned(),
        }
    }
}

/// Returns Rust playground url for the given code
pub async fn share(code: Code) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut req_json = HashMap::new();
    req_json.insert("code", &code.source_code);

    let client = reqwest::Client::new();
    let res: GistRes = client
        .post(GIST_GEN_URL)
        .json(&req_json)
        .send()
        .await?
        .json()
        .await?;

    let url = format!(
        "https://play.rust-lang.org/?version={}&mode={}&edition={}&gist={}",
        code.version, code.mode, code.edition, res.id
    );

    Ok(url)
}

/// Run the given code in Rust playground and return the output
pub async fn run(code: Code) -> Result<String, Box<dyn Error + Send + Sync>> {
    let req = RunReq::from(code);
    let client = reqwest::Client::new();
    let res: RunRes = client.post(RUN_URL).json(&req).send().await?.json().await?;

    Ok(format!("{}\n{}", res.stderr, res.stdout))
}
