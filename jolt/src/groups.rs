use serenity::framework::standard::macros::group;

use crate::commands::meta::*;
use crate::commands::moderation::*;

#[group]
#[commands(ping, info)]
struct General;

#[group]
#[commands(ban, kick, timeout)]
struct Moderators;