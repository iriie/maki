use serde;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::utils::html::clean_url;

const URBANDICTIONARY_API_URL: &str = "https://api.urbandictionary.com/v0/define?term={TERM}";
const RANDOM_PIKACHU_API_URL: &str = "https://uwucollective.cf/api/pika";

#[derive(Deserialize, Serialize, Debug)]
struct Definition {
    author: String,
    definition: String,
    example: String,
    defid: u64,
    permalink: String,
    thumbs_down: u64,
    thumbs_up: u64,
    word: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct UrbanResponse {
    #[serde(rename = "list")]
    pub definitions: Vec<Definition>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Image {
    pub image: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Gif {
    id: i64,
    url: String,
    category: String,
}

#[command]
#[description("whats a ship")]
async fn ship(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let first = args.single::<String>().unwrap();
    let second = args.single::<String>().unwrap();
    let to_compare: String = if &first > &second {
        first.to_owned() + &second
    } else {
        second.to_owned() + &first
    };
    let percent = divide_until(get_number_from_string(&to_compare).await, 100, 2).await;
    let exclamatory_message = match percent {
        0..=39 => "not too good.",
        40..=59 => "seems okay.",
        60..=68 => "nice!",
        69 => "nice.",
        70..=76 => "nice!",
        77..=97 => "woah! amazing!",
        98..=100 => "is this even possible?",
        _ => "this shouldnt happen???",
    };
    msg.channel_id
        .say(
            &ctx.http,
            format!(
                "shipping!!!\n1. {}\n2. {}\nboth of them seem to be {}% compatible! {}",
                first, second, percent, exclamatory_message
            ),
        )
        .await?;
    Ok(())
}

async fn get_number_from_string(string: &str) -> i32 {
    let bytes = string.as_bytes();
    let mut ret: i32 = 50;
    for b in 1..bytes.len() {
        ret = ret + bytes[b - 1] as i32;
    }
    ret
}

async fn divide_until(mut n: i32, until: i32, by: i32) -> i32 {
    while n > until {
        n = n / by
    }
    n
}

#[command]
#[aliases(ud)]
#[description("Gets definitions from UrbanDictionary")]
async fn urbandictionary(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if msg.channel(ctx).await.unwrap().guild().unwrap().is_nsfw() == false {
        return Err(CommandError::from(
            "This command must be run in an NSFW Channel.",
        ));
    }
    let data = get_data(&ctx, URBANDICTIONARY_API_URL, args.rest()).await?; //gets data from urbandictionary api

    let deserialized: UrbanResponse = serde_json::from_value(data.clone()).unwrap();

    let _check_for_word = if let Some(word) = data
        .pointer("/list/0/word") //where we get the data (in js would be list[0].word)
        .and_then(|x| x.as_str())
    //convert to string
    {
        word //assign to var
    } else {
        return Err(CommandError::from("Word not found")); //or return not found
    };

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(0x3498db);
                e.title(&format!("{}", deserialized.definitions[0].word.to_string()));
                e.field(
                    "Definition",
                    &deserialized.definitions[0].definition.to_string(),
                    true,
                );
                e.field(
                    "Example",
                    &deserialized.definitions[0].example.to_string(),
                    true,
                );
                e
            });
            m
        })
        .await?;

    Ok(())
}

#[command]
#[aliases(pika)]
async fn pikachu(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = get_data(&ctx, RANDOM_PIKACHU_API_URL, args.rest()).await?;

    let deserialized: Image = serde_json::from_value(data.clone()).unwrap();

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| e.color(0x3498db).title("pika!").image(deserialized.image))
        })
        .await?;
    Ok(())
}

#[command]
#[description("hug!")]
async fn hug(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let rest = args.remains();
    let title = match rest {
        Some(user) => match msg.mentions.len() >= 1 {
            true => {
                let to_be_called = match msg.clone().mentions[0]
                    .nick_in(&ctx.http, msg.guild_id.unwrap())
                    .await
                {
                    Some(nick) => nick,
                    None => msg.mentions[0].name.to_owned(),
                };
                format!("{} hugged {}!",msg.author_nick(&ctx.http).await.unwrap_or(msg.clone().author.name), to_be_called)
            }
            false => format!("{} hugged {}!",msg.author_nick(&ctx.http).await.unwrap_or(msg.clone().author.name), user),
        },
        None => "hugs!".to_string(),
    };
    let client = reqwest::Client::new();
    let results: Gif = client
        .get("https://gif.izu.moe/api/gif/hug")
        .send()
        .await?
        .json()
        .await?;

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| e.color(0x3498db).title(title).image(results.url))
        })
        .await?;
    Ok(())
}

#[command]
#[description("pat!")]
async fn pat(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let rest = args.remains();
    let title = match rest {
        Some(user) => match msg.mentions.len() >= 1 {
            true => {
                let to_be_called = match msg.clone().mentions[0]
                    .nick_in(&ctx.http, msg.guild_id.unwrap())
                    .await
                {
                    Some(nick) => nick,
                    None => msg.mentions[0].name.to_owned(),
                };
                format!("{} patted {}!",msg.author_nick(&ctx.http).await.unwrap_or(msg.clone().author.name), to_be_called)
            }
            false => format!("{} patted {}!",msg.author_nick(&ctx.http).await.unwrap_or(msg.clone().author.name), user),
        },
        None => "someone patted you!".to_string(),
    };

    let client = reqwest::Client::new();
    let results: Gif = client
        .get("https://gif.izu.moe/api/gif/pat")
        .send()
        .await?
        .json()
        .await?;

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| e.color(0x3498db).title(title).image(results.url))
        })
        .await?;
    Ok(())
}

async fn get_data(_ctx: &Context, url: &str, term: &str) -> Result<Value, CommandError> {
    let url = url.replace("{TERM}", &term);
    let url = clean_url(&url);

    let client = reqwest::Client::new();

    let resp = client.get(&url).send().await?.json().await?;

    Ok(resp)
}
