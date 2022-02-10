
<div align="center">
  <img width=300 src="https://rustacean.net/assets/rustacean-flat-happy.png">
  <h1>RPG_BOT (Rust Playground Bot)</h1>
  <p><a href="https://telegram.org/">Telegram</a> bot help you to run Rust code in <a href="https://telegram.org/">Telegram</a> via <a href="https://play.rust-lang.org/">Rust playground</a></p>
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

## Images
### Help Message
[![help_message](https://i.suar.me/nzYza/s)](https://i.suar.me/nzYza)

### Run Command
[![run_command](https://i.suar.me/aZ6mz/s)](https://i.suar.me/aZ6mz)

### Share Command
[![share_command](https://i.suar.me/289jQ/s)](https://i.suar.me/289jQ)

## License
The [GNU Affero General Public](https://www.gnu.org/licenses/agpl-3.0.en.html) License is a free, copyleft license for software and other kinds of works, specifically designed to ensure cooperation with the community in the case of network server software.
