use dotenv::dotenv;
use serde;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::env;

const GEOCODE_API_URL: &str =
    "http://dev.virtualearth.net/REST/v1/Locations/{SEARCH}?output=json&key={BING_MAPS_KEY}";
const DARK_SKY_API_URL: &str =
    "https://api.darksky.net/forecast/{DARK_SKY_KEY}/{LAT},{LONG}?exclude=daily,minutely,flags&units=us";

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
struct Geocode {
    #[serde(default)]
    copyright: String,
    resource_sets: Vec<ResourceSets>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ResourceSets {
    resources: Vec<Resources>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Resources {
    name: String,
    entity_type: String,
    point: Point,
}

#[derive(Deserialize, Serialize, Debug)]
struct Point {
    r#type: String,
    coordinates: Vec<f32>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DarkSky {
    #[serde(default)]
    latitude: f32,
    longitude: f32,
    currently: DSMainStruct,
    hourly: Hourly,
}

#[derive(Deserialize, Serialize, Debug)]
struct Hourly {
    summary: String,
    icon: String,
    data: Vec<DSMainStruct>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DSMainStruct {
    summary: String,
    icon: String,
    precip_intensity: f32,
    precip_probability: f32,
    temperature: f32,
    apparent_temperature: f32,
    dew_point: f32,
    humidity: f32,
    pressure: f32,
    wind_speed: f32,
    wind_gust: f32,
    wind_bearing: f32,
    cloud_cover: f32,
    uv_index: u32,
    visibility: f32,
    ozone: f32,
}

async fn get_data(url: String) -> Result<Value, CommandError> {
    println!("{:#?}", url);

    let client = reqwest::Client::new();

    let resp = client.get(&url).send().await?.json().await?;

    Ok(resp)
}

#[command]
#[aliases(w)]
async fn weather(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    dotenv().ok();
    let darksky_key =
        env::var("DARK_SKY_KEY").expect("Expected DARK_SKY_KEY to be set in environment");
    let bingmaps_key =
        env::var("BING_MAPS_KEY").expect("Expected BING_MAPS_KEY to be set in environment");

    let geocode_api_url_1: &str = &GEOCODE_API_URL
        .replace("{SEARCH}", args.rest())
        .replace("{BING_MAPS_KEY}", &bingmaps_key)
        .to_string();
    let data = get_data(geocode_api_url_1.to_string()).await?;
    let geocode_des: Geocode = serde_json::from_value(data.clone()).unwrap();
    if geocode_des.resource_sets[0].resources.len() == 0 {
        msg.channel_id
            .say(&ctx.http, "That place could not be found.").await?;
        return Err(CommandError::from("Place not found"));
    }

    let dark_sky_api_url_1: &str = &DARK_SKY_API_URL
        .replace(
            "{LAT}",
            &geocode_des.resource_sets[0].resources[0].point.coordinates[0].to_string(),
        )
        .replace(
            "{LONG}",
            &geocode_des.resource_sets[0].resources[0].point.coordinates[1].to_string(),
        )
        .replace("{DARK_SKY_KEY}", &darksky_key)
        .to_string();
    let data = get_data(dark_sky_api_url_1.to_string()).await?;
    let dark_sky_des: DarkSky = serde_json::from_value(data.clone()).unwrap();
    let footer = "Powered by Dark Sky - Hyperlocal Weather";

    let dirs = vec![
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];
    let emojis = vec!["🡡", "🡥", "🡢", "🡦", "🡣", "🡧", "🡠", "🡤"];

    let dir = ((dark_sky_des.currently.wind_bearing + 11.25) / 22.5) as usize;

    let wind_direction = format!(
        "{} ({}, {}°)",
        emojis[(dir / 2) % 7],
        dirs[dir % 15].to_owned(),
        dark_sky_des.currently.wind_bearing
    );

    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.color(0x3498db)
                .title(&format!(
                    "Weather for {}",
                    geocode_des.resource_sets[0].resources[0].name
                ))
                .url(&format!(
                    "https://darksky.net/forecast/{},{}",
                    geocode_des.resource_sets[0].resources[0].point.coordinates[0],
                    geocode_des.resource_sets[0].resources[0].point.coordinates[1]
                ))
                .thumbnail(&format!(
                    "https://darksky.net/images/weather-icons/{}.png",
                    dark_sky_des.currently.icon
                ))
                .description(&format!("{}", dark_sky_des.hourly.summary))
                .field(
                    "Temperature",
                    format!(
                        "Current: {}°F ({}°C)\nFeels Like: {}°F ({}°C)",
                        dark_sky_des.currently.temperature.round(),
                       ((dark_sky_des.currently.temperature - 32.0) * 5.0/9.0).round(),
                        dark_sky_des.currently.apparent_temperature.round(),
                        ((dark_sky_des.currently.apparent_temperature - 32.0) * 5.0/9.0).round()
                    ),
                    true,
                )
                .field(
                    "Precipitation",
                    format!("Chance: {}%", dark_sky_des.currently.precip_probability),
                    true,
                )
                .field(
                    "Etc.",
                    format!(
                        "Speed: {:.2}mph ({:.2}kph)\nDirection: {}",
                        dark_sky_des.currently.precip_probability, dark_sky_des.currently.precip_probability*1.609, wind_direction
                    ),
                    true,
                )
                .footer(|f| f.text(&format!("{}", footer)))
        })
    }).await;
    Ok(())
}
