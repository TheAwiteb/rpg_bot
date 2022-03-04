<div align="center">
  <img width=300 src="https://rustacean.net/assets/rustacean-flat-happy.png">
  <h1>RPG_BOT (Rust Playground Bot)</h1>
  <p><a href="https://telegram.org/">Telegram</a> bot help you to run Rust code in <a href="https://telegram.org/">Telegram</a> via <a href="https://play.rust-lang.org/">Rust playground</a></p>
  Languages we support it<br>
   ( AR 🇸🇦 - EN 🇺🇸 - RU 🇷🇺 )
   <br>
  <a href="https://github.com/TheAwiteb/rpg_bot#Add-new-language">Add New Language</a>
  <br><br>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/license-MIT-orange.svg" alt="License">
  </a>
  <a href="https://rust-lang.org/">
    <img src="https://img.shields.io/badge/Made%20with-Rust-orange.svg" alt="Rust">
  </a>
  <br>
  <a href="https://rust-reportcard.xuri.me/report/github.com/TheAwiteb/rpg_bot">
    <img src="https://rust-reportcard.xuri.me/badge/github.com/TheAwiteb/rpg_bot" alt="Rust report">
  </a>
  <br>
  <a href="https://github.com/theawiteb/rpg_bot">
    <img src="https://badge.fury.io/gh/theawiteb%2Frpg_bot.svg" alt="version">
  </a>
  <a href="https://github.com/TheAwiteb/rpg_bot/issues?q=is%3Aissue+is%3Aopen+">
    <img src="https://img.shields.io/github/issues/theawiteb/rpg_bot.svg" alt="issues-open">
  </a>
  <a href="https://github.com/TheAwiteb/rpg_bot/issues?q=is%3Aissue+is%3Aclosed+">
    <img src="https://img.shields.io/github/issues-closed/theawiteb/rpg_bot.svg" alt="issues-closed">
  </a>
</div>

<details open>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#Requirements">Requirements</a>
    </li>
    <li>
      <a href="#Bot-interface">Bot interface</a>
    </li>
    <li>
      <a href="#Features">Features</a>
    </li>
    <li>
      <a href="#Installation">Installation</a>
      <ul>
        <li>
          <a href="#Building">Building</a>
          <ul>
            <li>
              <a href="#With-Docker">With Docker</a>
            </li>
            <li>
              <a href="#With-Cargo">With Cargo</a>
            </li>
          </ul>
        </li>
        <li>
          <a href="#Running">Running</a>
          <ul>
            <li>
              <a href="#With-Docker">With Docker</a>
            </li>
            <li>
              <a href="#With-Cargo">With Cargo</a>
            </li>
          </ul>
        </li>
      </ul>
    </li>
    <li>
      <a href="#Add-new-language">Add new language</a>
    </li>
    <li>
      <a href="#Images">Images</a>
      <ul>
          <li><a href="#Help-Message">Help Message</a></li>
          <li><a href="#Run-Command">Run Command</a></li>
          <li><a href="#Share-Command">Share Command</a></li>
      </ul>
    </li>
    <li><a href="#License">License</a></li>
  </ol>
</details>

## Requirements
* With Cargo
  * [Rust](https://rust-lang.org/)
  * [Diesel CLI](https://crates.io/crates/diesel_cli)

* With Docker
  * [Docker](https://docker.com)

## Bot interface

The bot supports 3 straightforward commands:
- `/help <command (default: all)>` — help message for all or specified command.

- `/run <version (default: stable)> <mode (default: debug)> <edition (default: 2021)>` — use this command with reply to code you want to execute it.

- `/share <version (default: stable)> <mode (default: debug)> <edition (default: 2021)>` — use this command with reply to code you want to share it.

## Features
- Use [ORM](https://en.wikipedia.org/wiki/Object%E2%80%93relational_mapping) database with [Diesel](https://github.com/diesel-rs/diesel).
- Delay for each user.  <!-- (You can update it from bot) -->
- Beautiful telegram keyboard.
- Conditions that protect Rust Playground, including the inability to publish sources that are not in the Rust language, and also the inability to publish a source that was published in the same process (and with run as well).
- Delete the sources periodically (to prevent accumulation and increase in size).
- Languages support (You can [add new language](https://github.com/TheAwiteb/rpg_bot#Add-new-language)).
<!-- - Possibility to [Broadcast messages](https://www.dictionary.com/browse/broadcast) to all users 🤩 -->
<!-- - Do not save a previously saved source (the saved one is used). -->
<!-- - Admin interface -->
<!-- - Possibility to adjust the limit and the delay time for each user. -->
<!-- - Possibility to set more than one admin. -->

## Installation
### Building
You must put the [bot token](https://core.telegram.org/bots#3-how-do-i-create-a-bot) in the [environment file](.env) before building for it to be included

#### With Docker
```bash
git clone https://github.com/TheAwiteb/rpg_bot
cd rpg_bot
sudo docker build . -t rpg_bot
```
#### With Cargo
```bash
git clone https://github.com/TheAwiteb/rpg_bot
cd rpg_bot
diesel migration run
cargo build --release
```

### Running

#### With Docker
```bash
sudo docker run -ti --rm rpg_bot 
```
#### With Cargo
```bash
cargo run --release
```

## Add new language
To add a new language, it is very simple, just add a file with type [json](https://en.wikipedia.org/wiki/JSON) in the folder [i18n](i18n) that contains the sentences in the rest of the files and then do PR (preferably add the name of the new language in the [README.md](README.md) and also in the [`languages_ctx`](https://github.com/TheAwiteb/rpg_bot/blob/master/src/rpg_db.rs#L30) function, but don't worry, if you don't do this, we will do it)

## Images
### Help Message
[![help_message](https://i.suar.me/nzYza/s)](https://i.suar.me/nzYza)

### Run Command
[![run_command](https://i.suar.me/aZ6mz/s)](https://i.suar.me/aZ6mz)

### Share Command
[![share_command](https://i.suar.me/289jQ/s)](https://i.suar.me/289jQ)

### Languages
[![language_command](https://i.suar.me/aZ11z/s)](https://i.suar.me/aZ11z/s)

## License
The [GNU Affero General Public](https://www.gnu.org/licenses/agpl-3.0.en.html) License is a free, copyleft license for software and other kinds of works, specifically designed to ensure cooperation with the community in the case of network server software.
