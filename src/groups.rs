use serenity::framework::standard::macros::group;

use crate::commands::meta::*;
use crate::commands::moderation::*;

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(ban)]
struct Moderators;