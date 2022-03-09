// rpg_bot - Telegram bot 🤖, help you to run and share Rust code in Telegram via Rust playground 🦀
// Source code: <https://github.com/TheAwiteb/rpg_bot>
//
// Copyright (C) 2022 TheAwiteb <awiteb@hotmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::models::SourceCode;
use reqwest::Client;
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

impl From<SourceCode> for Code {
    fn from(source: SourceCode) -> Self {
        Self {
            source_code: source.source_code,
            version: source.version,
            edition: source.edition,
            mode: source.mode,
        }
    }
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

impl From<&Code> for RunReq {
    fn from(code: &Code) -> Self {
        Self {
            code: code.source_code.clone(),
            channel: code.version.clone(),
            mode: code.mode.clone(),
            edition: code.edition.clone(),
            ..Self::default()
        }
    }
}

impl RunRes {
    fn is_valid(&self) -> bool {
        !self
            .stderr
            .contains("error: could not compile `playground`")
    }
}

impl Code {
    pub fn new(source_code: &str, version: &str, mode: &str, edition: &str) -> Self {
        Self {
            source_code: source_code.to_string(),
            version: version.to_string(),
            mode: mode.to_string(),
            edition: edition.to_string(),
        }
    }

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

async fn get_run_response(code: &Code) -> Result<RunRes, Box<dyn Error + Send + Sync>> {
    let body = RunReq::from(code);
    Ok(Client::new()
        .post(RUN_URL)
        .json(&body)
        .send()
        .await?
        .json()
        .await?)
}

async fn get_share_response(code: &Code) -> Result<GistRes, Box<dyn Error + Send + Sync>> {
    let mut req_json = HashMap::new();
    req_json.insert("code", &code.source_code);
    Ok(Client::new()
        .post(GIST_GEN_URL)
        .json(&req_json)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap())
}

/// Returns Rust playground url for the given code
pub async fn share(code: &Code) -> Result<String, String> {
    let res_run: RunRes = get_run_response(code)
        .await
        .map_err(|err| format!("{}", err))?;

    if res_run.is_valid() {
        let res_share: GistRes = get_share_response(code)
            .await
            .map_err(|err| format!("{}", err))?;
        let url = format!(
            "https://play.rust-lang.org/?version={}&mode={}&edition={}&gist={}",
            code.version, code.mode, code.edition, res_share.id
        );

        Ok(url)
    } else {
        Err(res_run
            .stderr
            .replace(
                "could not compile `playground`",
                "Source code cannot be shared",
            )
            .replace("/playground", "playground"))
    }
}

/// Run the given code in Rust playground and return the output
pub async fn run(code: &Code) -> Result<String, String> {
    let res: RunRes = get_run_response(code)
        .await
        // Returns error as string for send it to user
        .map_err(|err| format!("{}", err))?;

    let output: String = format!(
        "{}\n{}",
        res.stderr.replace("/playground", "playground"),
        res.stdout
    );

    if res.is_valid() {
        Ok(output)
    } else {
        Err(output)
    }
}
