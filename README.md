## RPG_BOT (Rust Playground Bot)

[![LICENSE](https://img.shields.io/badge/license-MIT-orange.svg)](./LICENSE)

Telegram bot help you to run Rust code in Telegram via Rust playground

## Bot interface

The bot supports 3 straightforward commands:
- `/help <command (default: all)>` — help message for all or specified command.

- `/run <version (default: stable)> <mode (default: debug)> <edition (default: 2021)>` — use this command with reply to code you want to execute it.

- `/share <version (default: stable)> <mode (default: debug)> <edition (default: 2021)>` — use this command with reply to code you want to share it.

## Building
You must put the [bot token](https://core.telegram.org/bots#3-how-do-i-create-a-bot) in the [environment file](.env) before building for it to be included

### With Docker
```bash
git clone https://github.com/TheAwiteb/rpg_bot
cd rpg_bot
sudo docker build . -t rpg_bot
```
### With Cargo
```bash
git clone https://github.com/TheAwiteb/rpg_bot
cd rpg_bot
cargo build --release
```

## Running

### With Docker
```bash
sudo docker run -ti --rm rpg_bot 
```
### With Cargo
```bash
cargo run --release
```

## Images
### Help Message
![help_message](https://i.suar.me/78r8G/s)

### Run Command
![run_command](https://i.suar.me/VNlN9/s)

### Share Command
![share_command](https://i.suar.me/wg0gp/s)

## License
RPG_BOT is under the MIT license. See the [LICENSE](LICENSE) file for details.
