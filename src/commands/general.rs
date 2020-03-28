use log::error;

use serde_json::Value;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

const GEOCODE_API_URL: &str = "https://www.bing.com/api/v6/Places/AutoSuggest?q={SEARCH}&appid=D41D8CD98F00B204E9800998ECF8427E1FBE79C2&structuredaddress=true&types=&setmkt=en-GB&clientid=308E26AC3B4C607B382C2B343A606114";

cached! {
    CACHE_DATA;
    fn get_data(url: &'static str, search: String) -> Result<Value, CommandError> = {
        let url = url.replace("{SEARCH}", &search);
        println!("{:#?}", url);

        // fetch data
        let client = reqwest::blocking::Client::new();
    
        match client.get(&url).send().and_then(|x| x.json()) {
            //get from url and convert to json
            Ok(val) => Ok(val), //send data back as serde_json value
            Err(e) => {
                error!("[GRL] Failed to fetch data: {}", e);
                Err(CommandError::from(&format!("Failed to get data from given URL: {}", url)))
            }
        }
    }
}

#[command]
#[aliases(w)]
fn weather(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = get_data(GEOCODE_API_URL, args.rest().to_string())?;
    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.color(0x3498db)
                .title(&format!("{:#?}", ::serde_json::to_string_pretty(&data)))
        })
    });
    Ok(())
}