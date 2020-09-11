use aspotify::{Client, ClientCredentials, ItemType};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[aliases(s, sp, spot)]
#[description("Gets things from Spotify. Defaults to \"songs\".\nSubcommands: `songs`")]
#[sub_commands(SPOTIFY_SONGS, SPOTIFY_ALBUM)]
async fn spotify(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    spotify_songs(ctx, msg, args).await?;
    Ok(())
}

#[command("songs")]
#[aliases(s)]
async fn spotify_songs(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let credentials =
        ClientCredentials::from_env().expect("CLIENT_ID and CLIENT_SECRET not found.");

    // Create a Spotify client.
    let spoticlient = Client::new(credentials);

    let result = spoticlient
        .search()
        .search(
            args.rest(),
            [ItemType::Track].iter().copied(),
            false,
            1,
            0,
            None,
        )
        .await?;

    debug!("{:#?}", result);

    if result.clone().data.tracks.unwrap().items.len() == 0 {
        msg.channel_id
            .say(&ctx.http, "No songs were found that matched the input.")
            .await?;
        return Err(CommandError::from("h-No songs were found."));
    }

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            let desc = "by ".to_string()
                + &result.clone().data.tracks.unwrap().items[0].artists[0]
                    .name
                    .to_string()
                + "\non "
                + &result.clone().data.tracks.unwrap().items[0]
                    .album
                    .name
                    .to_string()
                + "\n[view on Spotify >]("
                + &result.clone().data.tracks.unwrap().items[0].external_urls["spotify"]
                    .to_string()
                + ")";

            m.embed(|e| {
                e.author(|a| {
                    a.name(&format!(
                        "{}",
                        &result.clone().data.tracks.unwrap().items[0].name
                    ))
                    .url(
                        &result.clone().data.tracks.unwrap().items[0].external_urls["spotify"]
                            .to_string(),
                    )
                })
                .color(0xb90000)
                .description(desc)
                .thumbnail(&result.clone().data.tracks.unwrap().items[0].album.images[0].url)
                .footer(|f| f.text(format!("Data from Spotify Web API")))
            })
        })
        .await;

    Ok(())
}

#[command("albums")]
#[aliases(a)]
async fn spotify_album(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let credentials =
        ClientCredentials::from_env().expect("CLIENT_ID and CLIENT_SECRET not found.");

    // Create a Spotify client.
    let spoticlient = Client::new(credentials);

    let result = spoticlient
        .search()
        .search(
            args.rest(),
            [ItemType::Album].iter().copied(),
            false,
            1,
            0,
            None,
        )
        .await?;

    if result.clone().data.albums.unwrap().items.len() == 0 {
        msg.channel_id
            .say(&ctx.http, "No albums were found that matched the input.")
            .await?;
        return Err(CommandError::from("h-No albums were found."));
    }

    let album = spoticlient
        .albums()
        .get_album(
            &result.clone().data.albums.unwrap().items[0]
                .id
                .as_deref()
                .unwrap(),
            None,
        )
        .await?;

    dbg!(album.clone().data.tracks.total);

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            let desc = "by ".to_string()
                + &result.clone().data.albums.unwrap().items[0].artists[0]
                    .name
                    .to_string()
                + "\n"
                + &album.clone().data.tracks.total.to_string()
                + " tracks\n[view on Spotify >]("
                + &result.clone().data.albums.unwrap().items[0].external_urls["spotify"]
                    .to_string()
                + ")";

            m.embed(|e| {
                e.author(|a| {
                    a.name(&format!(
                        "{}",
                        &result.clone().data.albums.unwrap().items[0].name
                    ))
                    .url(
                        &result.clone().data.albums.unwrap().items[0].external_urls["spotify"]
                            .to_string(),
                    )
                })
                .color(0xb90000)
                .description(desc)
                .thumbnail(&result.clone().data.albums.unwrap().items[0].images[0].url)
                .footer(|f| f.text(format!("Data from Spotify Web API")))
            })
        })
        .await;

    Ok(())
}
