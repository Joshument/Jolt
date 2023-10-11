# Jolt
Jolt is a new, up-and-coming discord bot designed to replace most general-purpose and moderation bots. Discord bots have often been distributed as closed-source, along with premium features, meaning that you are forced to share bandwith with other people as well as pay for features. With Jolt, you are free to host your own instance of the bot, and any necessary premium features will always be open for you to use on your own self-host.

Jolt is currently in an incomplete state, however, it is in active development. Expect to see more features very soon.

## Features
- Full support for slash command syntax as well as prefix syntax
- Fully functional moderation system using unix timestamps to better display dates and times
- Support for discord's timeout feature - no more using a mute role if you don't want to
- Log your moderations in a moderation channel!

## Up-and-coming features
- Beautify your rules and announcements using embeds via an easy to use command
- Automatically assign roles on member join
- Unlimited reaction roles in any channel at any time
- ~~Robust auto-moderation with full regex support~~ thanks for taking my idea discord
- Allow server boosters to create their own custom roles
- XP system with the ability to fine-tune however you like, up to and including being able to define your own XP curves
- Auto-prune inactive members with role addition, removal, and kicks
- Create your own custom commands and scheduled events to be run

## Commands
As of now, these are the current supported commands:

### Meta
- ping
- info
### Moderation
- warn \<user> [reason]
- delwarn \<id>
- warnings \<user> [page]
- timeout \<user> \<time> [reason]
- untimeout \<user> [reason]
- mute \<user> [time] [reason]
- unmute \<user> [reason]
- kick \<user> [reason]
- ban \<user> [time] [reason]
- unban \<user> [reason]
- modlogs \<user> [page]
### Configuration
- muterole \<role>
- logschannel \<channel>
- setprefix \<prefix>
- setup

## Local Setup
finish this later!! for now this is more a reminder to myself, but:
1. make sure you have sqlx-cli installed via `cargo install sqlx-cli`
2. `sqlx db create`
3. `sqlx migrate run`
4. `cargo run`

## Adding your own modifications
Jolt uses the [poise](https://docs.rs/poise/latest/poise/) crate as a framework. If you would like to add commands, it's as simple as linking your functions to the framework in `main`! Support for primitive custom commands will be added in the future if you are not a rust programmer.

## Issues
Please report any issues to the [issue tracker](https://github.com/Joshument/Jolt/issues).

## License
Jolt is licensed under the [BSD 3-Clause License](https://github.com/Joshument/Jolt/blob/main/LICENSE). If you do not want to read the license, tl;dr do not use the names of any contributors of this bot to promote or advertise you derivitave work (you are allowed to mention us).
