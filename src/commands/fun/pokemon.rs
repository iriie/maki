use rand::Rng;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PokemonResults {
    hits: Vec<Hit>,
    offset: i64,
    limit: i64,
    #[serde(rename = "nbHits")]
    nb_hits: i64,
    #[serde(rename = "exhaustiveNbHits")]
    exhaustive_nb_hits: bool,
    #[serde(rename = "processingTimeMs")]
    processing_time_ms: i64,
    query: String,
}

#[derive(Serialize, Deserialize)]
pub struct Hit {
    species: String,
    num: i64,
    types: Vec<String>,
    #[serde(rename = "genderRatio")]
    gender_ratio: GenderRatio,
    #[serde(rename = "baseStats")]
    base_stats: BaseStats,
    abilities: Abilities,
    heightm: f64,
    weightkg: f64,
    color: String,
    #[serde(rename = "eggGroups")]
    egg_groups: Vec<String>,
    image: String,
    #[serde(rename = "dexId")]
    dex_id: i64,
    id: i64,
    sprite: String,
    #[serde(rename = "flavorText")]
    flavor_text: Vec<FlavorText>,
    evos: Option<Vec<String>>,
    #[serde(rename = "baseForme")]
    base_forme: Option<String>,
    #[serde(rename = "cosmeticFormes")]
    cosmetic_formes: Option<Vec<String>>,
    #[serde(rename = "baseSpecies")]
    base_species: Option<String>,
    forme: Option<String>,
    #[serde(rename = "formeLetter")]
    forme_letter: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Abilities {
    first: String,
    hidden: Option<String>,
    second: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BaseStats {
    hp: i64,
    atk: i64,
    def: i64,
    spa: i64,
    spd: i64,
    spe: i64,
}
#[derive(Serialize, Deserialize)]
pub struct FlavorText {
    version_id: String,
    flavor_text: String,
}

#[derive(Serialize, Deserialize)]
pub struct GenderRatio {
    male: f64,
    female: f64,
}

#[command]
#[aliases(poke, pk, pokemon)]
#[description(
    "Gets things from Favna's Pokemon API. Defaults to \"pokedex\".\nSubcommands: `pokedex`"
)]
#[sub_commands(POKEMON_POKEDEX)]
async fn pokemon(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    pokemon_pokedex(ctx, msg, args).await?;
    Ok(())
}
#[command("pokedex")]
#[aliases(dex)]
async fn pokemon_pokedex(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let pokemon: String = args.rest().to_string(); // gets args

    let client = reqwest::Client::new();

    let pokemon_results: PokemonResults = client
        .get(&format!(
            "https://dex.izu.moe/api/search/full?q={}",
            pokemon
        ))
        .send()
        .await?
        .json()
        .await?;

    if pokemon_results.hits.len() < 1 {
        msg.channel_id
            .say(
                &ctx.http,
                format!("couldn't find anything for {}.", pokemon),
            )
            .await?;
        return Ok(());
    }
    let poke = &pokemon_results.hits[0];

    let num: usize = rand::thread_rng().gen_range(0, &poke.flavor_text.len());

    let pokemon_types = &poke.types.join("/"); // if more than one type, combines them with a "/"

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(0x3498db)
                    .title(format!(
                        "Pokédex: {} | {}",
                        cap_first_letter(&poke.species),
                        poke.num
                    ))
                    .url(format!(
                        "https://www.pokemon.com/us/pokedex/{}",
                        &poke.species.replace(" ", "-")
                    ))
                    .thumbnail(format!(
                        "https://assets.pokemon.com/assets/cms2/img/pokedex/full/{:03}.png",
                        poke.num
                    ))
                    .description(format!(
                        "Types: {}\nHeight: {}m\nWeight: {}kg\n\n{}\n- Pokémon {}",
                        pokemon_types,
                        &poke.heightm,
                        &poke.weightkg,
                        &poke.flavor_text[num].flavor_text,
                        &poke.flavor_text[num].version_id
                    ))
                    .footer(|f| f.text("Data from dex.izu.moe"))
            })
        })
        .await;

    Ok(())
}
fn cap_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
