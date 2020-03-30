use log::error;
use serde_json::Value;
use serde;
use serde::{Serialize, Deserialize};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::utils::html::clean_url;

const URBANDICTIONARY_API_URL: &str = "https://api.urbandictionary.com/v0/define?term={TERM}";
const RANDOM_PIKACHU_API_URL: &str = "https://some-random-api.ml/pikachuimg";

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
struct Response {
    #[serde(rename = "list")]
    pub definitions: Vec<Definition>,
    pub tags: Option<Vec<String>>,
}

#[command]
#[aliases(ud)]
#[description("Gets definitions from UrbanDictionary")]
fn urbandictionary(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = get_data(&ctx, URBANDICTIONARY_API_URL, args.rest())?; //gets data from urbandictionary api

    let deserialized: Response = serde_json::from_value(data.clone()).unwrap();

    println!("{:#?}", deserialized);

    let _check_for_word = if let Some(word) = data
        .pointer("/list/0/word") //where we get the data (in js would be list[0].word)
        .and_then(|x| x.as_str())
    //convert to string
    {
        word //assign to var
    } else {
        return Err(CommandError::from("Word not found")); //or return not found
    };

    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.color(0x3498db)
                .title(&format!("{}", deserialized.definitions[0].word))
                .field("Definition", &deserialized.definitions[0].definition, true)
                .field("Example", &deserialized.definitions[0].example, true)
        })
    });

    Ok(())
}

#[command]
#[aliases(pika)]
fn pikachu(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = get_data(&ctx, RANDOM_PIKACHU_API_URL, args.rest())?;
    let img = data
        .pointer("/link") //where we get the data (in js would be list[0].link)
        .and_then(|x| x.as_str()) //convert to string
        .unwrap_or("N/A"); //if not available, set var as "N/A"
    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| e.color(0x3498db).title("pika!").image(img))
    });
    Ok(())
}

fn get_data(_ctx: &Context, url: &str, term: &str) -> Result<Value, CommandError> {
    let url = url.replace("{TERM}", &term);
    let url = clean_url(&url);
    println!("{:#?}", url);
    // fetch data

    let client = reqwest::blocking::Client::new();

    match client.get(&url).send().and_then(|x| x.json()) {
        //get from url and convert to json
        Ok(val) => Ok(val), //send data back as serde_json value
        Err(e) => {
            error!("[GRP:fun] Failed to fetch data: {}", e);
            Err(CommandError::from(&format!(
                "Failed to get data from given URL{}",
                url
            )))
        }
    }
}