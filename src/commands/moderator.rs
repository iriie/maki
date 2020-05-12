use serenity::framework::standard::macros::command;
use serenity::{
    framework::standard::{Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[description("Prunes messages. (limit 99 at a time)")]
async fn prune(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}