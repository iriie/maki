use log::error;

use serde_json::Value;
use serde;
use serde::{Serialize, Deserialize};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

static POKEMON_API_URL: &str = "https://pokeapi.co/api/v2/{ENDPOINT}/{TERM}";

#[derive(Deserialize, Serialize, Debug)]
struct FlavorTextEntries {
    flavor_text: String,
    language: Language,
    version: Version,
}

#[derive(Deserialize, Serialize, Debug)]
struct Names {
    name: String,
    language: Language,
}

#[derive(Deserialize, Serialize, Debug)]
struct Types {
    slot: i32,
    r#type: Type,
}

#[derive(Deserialize, Serialize, Debug)]
struct Type {
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Language {
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Version {
    name: String,
    url: String
}

#[derive(Deserialize, Serialize, Debug)]
struct PokemonSpecies {
    #[serde(default)]
    flavor_text_entries: Vec<FlavorTextEntries>,
    names: Vec<Names>,
    name: String,
    id: i32,
}

#[derive(Deserialize, Serialize, Debug)]
struct Pokemon {
    #[serde(default)]
    types: Vec<Types>,
    id: i32,
    height: i32,
    weight: i32,
}

cached! {
    CACHE_DATA;
    fn get_data(url: &'static str, endpoint: &'static str, term: String) -> Result<Value, CommandError> = {
        let url = url.replace("{ENDPOINT}", &endpoint);
        let url = url.replace("{TERM}", &term);
        println!("{:#?}", url);

        // fetch data
        let client = reqwest::blocking::Client::new();
    
        match client.get(&url).send().and_then(|x| x.json()) {
            //get from url and convert to json
            Ok(val) => Ok(val), //send data back as serde_json value
            Err(e) => {
                error!("[GRP:pokedex] Failed to fetch data: {}", e);
                Err(CommandError::from(&format!("Failed to get data from given URL: {}", url)))
            }
        }
    }
}

#[command]
#[aliases(poke, pk, pokemon)]
fn pokedex(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let text = 2;
    let pokemon_species_json = get_data(POKEMON_API_URL, "pokemon-species", args.rest().to_string())?;
    let pokemon_species_des: PokemonSpecies = serde_json::from_value(pokemon_species_json.clone()).unwrap();
    let pokemon_json = get_data(POKEMON_API_URL, "pokemon", args.rest().to_string())?;
    let pokemon_des: Pokemon = serde_json::from_value(pokemon_json.clone()).unwrap();

    println!("{:#?}", pokemon_des);

    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.color(0x3498db)
                .title(format!("Pokédex: {} | {}", cap_first_letter(&pokemon_species_des.name), pokemon_des.id))
                    .url(format!("https://www.pokemon.com/us/pokedex/{}", pokemon_species_des.name))
                .thumbnail(format!("https://assets.pokemon.com/assets/cms2/img/pokedex/full/{:03}.png",pokemon_des.id))
                .description(&pokemon_species_des.flavor_text_entries[text].flavor_text)
        })
    });
    Ok(())
}

fn cap_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}