
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[aliases(s, sp, spot)]
#[description(
    "Gets things from Spotify. Defaults to \"songs\".\nSubcommands: `songs`"
)]
#[sub_commands(SPOTIFY_SONGS)]
async fn spotify(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let _cool = spotify_songs(ctx, msg, args);
    Ok(())
}

#[command("songs")]
#[aliases(s)]
async fn spotify_songs(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}