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

    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| e.color(0x3498db).title("pika!").image(deserialized.image))
    }).await?;
    Ok(())
}

async fn get_data(_ctx: &Context, url: &str, term: &str) -> Result<Value, CommandError> {
    let url = url.replace("{TERM}", &term);
    let url = clean_url(&url);

    let client = reqwest::Client::new();

    let resp = client.get(&url).send().await?.json().await?;

    Ok(resp)
}
