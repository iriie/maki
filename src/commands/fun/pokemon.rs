use graphql_client::{GraphQLQuery, Response};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;
use rand::Rng;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/pokemon_schema.graphql",
    query_path = "graphql/pokemon_query.graphql",
    response_derives = "Debug"
)]
struct MyQuery;

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
    let pokemon: String = args.rest().to_string();

    let q = MyQuery::build_query(my_query::Variables { pokemon: pokemon });

    let client = reqwest::Client::new();

    let res = client.post("https://favware.tech/api").json(&q).send().await?;

    let response_body: Response<my_query::ResponseData> = res.json().await?;

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
        msg.channel_id.say(&ctx.http, "That pokemon could not be found.").await?;
        return Err(CommandError::from(
            "Pokemon not found",
        ))
    }

    let response_data: my_query::ResponseData = response_body.data.expect("missing response data");

    let num = rand::thread_rng().gen_range(0, &response_data.get_pokemon_details_by_fuzzy.flavor_texts.len());
    let pokemon_types = &response_data.get_pokemon_details_by_fuzzy.types.join("/");
    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.color(0x3498db)
                .title(format!(
                    "Pokédex: {} | {}",
                    cap_first_letter(&response_data.get_pokemon_details_by_fuzzy.species),
                    response_data.get_pokemon_details_by_fuzzy.num
                ))
                .url(format!(
                    "https://www.pokemon.com/us/pokedex/{}",
                    &response_data.get_pokemon_details_by_fuzzy.species.replace(" ", "-")
                ))
                .thumbnail(format!(
                    "https://assets.pokemon.com/assets/cms2/img/pokedex/full/{:03}.png",
                    response_data.get_pokemon_details_by_fuzzy.num
                ))
                .description(format!(
                    "Types: {}\nHeight: {}m\nWeight: {}kg\n\n{}\n- Pokémon {}",
                    pokemon_types,
                    &response_data.get_pokemon_details_by_fuzzy.height,
                    &response_data.get_pokemon_details_by_fuzzy.weight,
                    &response_data.get_pokemon_details_by_fuzzy.flavor_texts[num].flavor,
                    &response_data.get_pokemon_details_by_fuzzy.flavor_texts[num].game
                ))
                .footer(|f| f.text("Data from favware/graphql-pokemon"))
        })
    }).await;

    Ok(())
}
fn cap_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
