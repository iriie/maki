use log::error;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::utils::html::clean_url;
use chrono::naive::NaiveDateTime;
use chrono::Utc;
use serde_json::Value;

const FM_RECENT_TRACKS_URL: &str = "http://ws.audioscrobbler.com/2.0/?method=user.getRecentTracks&user={USER}&api_key={KEY}&format=json&limit=10";
const FM_TOP_TRACKS_URL: &str = "http://ws.audioscrobbler.com/2.0/?method=user.gettoptracks&user={USER}&api_key={KEY}&format=json&limit=10&period={PERIOD}";

#[command]
#[aliases(fm)]
#[description(
    "Gets latest things from last.fm. Defaults to \"latest\".\nSubcommands: `latest`, `topsongs`, `latestsongs`"
)]
#[sub_commands(LASTFM_LATEST, LASTFM_TOPSONGS, LASTFM_LATESTSONGS, LASTFM_SAVE)]
fn lastfm(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = get_lastfm_data(&ctx, FM_RECENT_TRACKS_URL, args.rest(), "0")?;
    println!("{:#?}", args.rest());
    recent_track(ctx, msg, &data, false)?;

    Ok(())
}
#[command("save")]
#[aliases(update)]
fn lastfm_save(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    save_lastfm_username(ctx, msg, msg.author.id.0, &args);

    Ok(())
}
#[command("latest")]
fn lastfm_latest(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = get_lastfm_data(&ctx, FM_RECENT_TRACKS_URL, args.rest(), "0")?;
    println!("{:#?}", args.rest());
    recent_track(ctx, msg, &data, false)?;

    Ok(())
}

#[command("topsongs")]
fn lastfm_topsongs(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = get_lastfm_data(&ctx, FM_TOP_TRACKS_URL, args.rest(), "0")?;

    top_tracks(ctx, msg, &data, "0");

    Ok(())
}
#[command("latestsongs")]
#[aliases(songs)]
fn lastfm_latestsongs(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = get_lastfm_data(&ctx, FM_RECENT_TRACKS_URL, args.rest(), "0")?;

    recent_tracks(ctx, msg, &data);

    Ok(())
}

fn recent_track(
    ctx: &mut Context,
    msg: &Message,
    data: &Value,
    saved: bool,
) -> Result<(), CommandError> {
    let username = if let Some(username) = data
        .pointer("/recenttracks/@attr/user")
        .and_then(|x| x.as_str())
    {
        username
    } else {
        return Err(CommandError::from("last.fm profile not found"));
    };

    let last_track_artist = data
        .pointer("/recenttracks/track/0/artist/#text")
        .and_then(|x| x.as_str())
        .unwrap_or("N/A");
    let last_track_name = data
        .pointer("/recenttracks/track/0/name")
        .and_then(|x| x.as_str())
        .unwrap_or("N/A");
    let last_track_album = data
        .pointer("/recenttracks/track/0/album/#text")
        .and_then(|x| x.as_str())
        .unwrap_or("N/A");
    let last_track_url = data
        .pointer("/recenttracks/track/0/url")
        .and_then(|x| x.as_str())
        .unwrap_or("https://www.last.fm");

    // urlencode parenthesis
    let last_track_url = clean_url(last_track_url);

    // check for empty values that break embeds
    let username = if username.is_empty() { "N/A" } else { username };

    let last_track_artist = if last_track_artist.is_empty() {
        "N/A"
    } else {
        last_track_artist
    };

    let last_track_name = if last_track_name.is_empty() {
        "N/A"
    } else {
        last_track_name
    };

    let last_track_album = if last_track_album.is_empty() {
        "N/A"
    } else {
        last_track_album
    };

    // default blank image for fallback
    let last_track_image = {
        let img = data
            .pointer("/recenttracks/track/0/image/2/#text")
            .and_then(|x| x.as_str())
            .unwrap_or("");

        if img.is_empty() {
            "https://i.imgur.com/oYm77EU.jpg"
        } else {
            img
        }
    };

    // get the last track timestamp,
    // if it's currently playing, use now timestamp
    let last_track_timestamp = data
        .pointer("/recenttracks/track/0/date/uts")
        .and_then(|x| x.as_str())
        .and_then(|x| x.parse::<i64>().ok())
        .and_then(|x| Some(NaiveDateTime::from_timestamp(x, 0)))
        .unwrap_or_else(|| Utc::now().naive_utc())
        .format("%Y-%m-%dT%H:%M:%S");

    let last_track_status = if let Some(nowplaying) = data
        .pointer("/recenttracks/track/0/@attr/nowplaying")
        .and_then(|x| x.as_str())
    {
        if nowplaying == "true" {
            "now playing"
        } else {
            "latest track"
        }
    } else {
        "latest track"
    };

    let total_tracks = data
        .pointer("/recenttracks/@attr/total")
        .and_then(|x| x.as_str())
        .unwrap_or("N/A");

    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        let mut m = m;

        if saved {
            m = m.content("kanb");
        }

        let desc = "[".to_string()
            + &last_track_name.to_string()
            + "]("
            + &last_track_url.to_string()
            + ") by "
            + &last_track_artist.to_string()
            + "\non "
            + &last_track_album.to_string()
            + "\n[view on last.fm >]("
            + &last_track_url.to_string()
            + ")";

        m.embed(|e| {
            e.author(|a| {
                a.name(&format!("{}'s {}", username, last_track_status))
                    .url(&format!("https://www.last.fm/user/{}", username))
            })
            .color(0xb90000)
            .description(desc)
            .thumbnail(last_track_image)
            .footer(|f| f.text(format!("Total Tracks: {}", total_tracks)))
            .timestamp(last_track_timestamp.to_string())
        })
    });

    Ok(())
}

fn top_tracks(ctx: &mut Context, msg: &Message, data: &Value, period: &str) {
    let username = data
        .pointer("/recenttracks/@attr/user")
        .and_then(|x| x.as_str())
        .unwrap_or("N/A");
    let default_vec = vec![];
    let tracks = data
        .pointer("/recenttracks/track")
        .and_then(|x| x.as_array())
        .unwrap_or(&default_vec);

    if tracks.is_empty() {
        let _ = msg.channel_id.say(
            &ctx.http,
            "No recent tracks found. Go listen to some stuff!",
        );
        return;
    }

    let mut s = String::new();

    let first_image = tracks
        .first()
        .and_then(|x| x.pointer("/image/2/#text"))
        .and_then(|x| x.as_str())
        .unwrap_or("N/A");

    for (i, track) in tracks.iter().enumerate() {
        let title = track
            .pointer("/name")
            .and_then(|x| x.as_str())
            .unwrap_or("N/A");
        let url = track
            .pointer("/url")
            .and_then(|x| x.as_str())
            .unwrap_or("N/A");
        let artist = track
            .pointer("/artist/name")
            .and_then(|x| x.as_str())
            .unwrap_or("N/A");
        let topush: String = "`[".to_string()
            + &(i + 1).to_string()
            + "]` **["
            + title
            + "]("
            + &clean_url(url)
            + ")** by"
            + artist
            + "\n";

        let _ = s.push_str(&topush);
    }
    let title = "".to_string() + &username + "'s top tracks - " + &period;
    send_last_fm_embed(ctx, msg, None, &title, username, &s, first_image);
}

fn recent_tracks(ctx: &mut Context, msg: &Message, data: &Value) {
    let username = data
        .pointer("/recenttracks/@attr/user")
        .and_then(|x| x.as_str())
        .unwrap_or("N/A");
    let default_vec = vec![];
    let tracks = data
        .pointer("/recenttracks/track")
        .and_then(|x| x.as_array())
        .unwrap_or(&default_vec);

    if tracks.is_empty() {
        let _ = msg.channel_id.say(&ctx.http, "No top tracks found.");
        return;
    }

    let mut s = String::new();

    let first_image = tracks
        .first()
        .and_then(|x| x.pointer("/image/2/#text"))
        .and_then(|x| x.as_str())
        .unwrap_or("N/A");

    for (i, track) in tracks.iter().enumerate() {
        let title = track
            .pointer("/name")
            .and_then(|x| x.as_str())
            .unwrap_or("N/A");
        let url = track 
            .pointer("/url")
            .and_then(|x| x.as_str())
            .unwrap_or("N/A");
        let artist = track
            .pointer("/artist/#text")
            .and_then(|x| x.as_str())
            .unwrap_or("N/A");

        let topush: String = "`[".to_string()
            + &(i + 1).to_string()
            + "]` "
            + "**["
            + title
            + "]("
            + &clean_url(url)
            + ")** by "
            + artist
            + "\n";

        let _ = s.push_str(&topush);
    }
    let title = "".to_string() + &username + "'s latest tracks";
    send_last_fm_embed(ctx, msg, None, &title, username, &s, first_image);
}

fn save_lastfm_username(ctx: &mut Context, msg: &Message, user: u64, args: &Args){
    let tosay = "".to_string() + &msg.author.tag() + "(" + &user.to_string() + ")" + "'s last.fm username will be saved as " + args.rest();
    let _ = msg.channel_id.say(&ctx.http, tosay);
}

fn get_lastfm_data(
    _ctx: &Context,
    url: &str,
    username: &str,
    period: &str,
) -> Result<Value, CommandError> {
    println!("{:#?}", username);
    let fm_key = "a0eed701b74a0eac6dd9a13cdff63a9c";
    let url = url
        .replace("{USER}", &username)
        .replace("{KEY}", &fm_key)
        .replace("{PERIOD}", period);
    println!("{:#?}", url);
    // fetch data

    let client = reqwest::blocking::Client::new();

    match client.get(&url).send().and_then(|x| x.json()) {
        Ok(val) => Ok(val),
        Err(e) => {
            error!("[CMD:fm] Failed to fetch last.fm data: {}", e);
            Err(CommandError::from("Failed to get last.fm data."))
        }
    }
}

fn send_last_fm_embed(
    ctx: &Context,
    msg: &Message,
    content: Option<&str>,
    title: &str,
    username: &str,
    desc: &str,
    thumbnail: &str,
) {
    let split_desc = desc.split("\n");
    let mut count = 0;
    let mut truncated_desc = String::new();

    for line in split_desc {
        if count + line.len() >= 2000 {
            break;
        }
        truncated_desc = format!("{}\n{}", truncated_desc, line);
        count = truncated_desc.len();
    }

    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        let mut m = m;

        if let Some(content) = content {
            m = m.content(content);
        }

        m.embed(|e| {
            e.author(|a| {
                a.name(title)
                    .url(&format!("https://www.last.fm/user/{}", username))
            })
            .color(0xb90000)
            .description(&truncated_desc)
            .thumbnail(thumbnail)
        })
    });
}
