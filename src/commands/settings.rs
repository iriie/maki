use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use regex::Regex;

use serde;
use serde::{Deserialize, Serialize};

use crate::dynamic_prefix;
use crate::keys::ConnectionPool;
use crate::utils::user::get_members;
use sqlx;

#[derive(Deserialize, Serialize, Debug)]
struct UpdatePronoun {
    id: i64,
    pronouns: Option<String>,
}
#[derive(Deserialize, Serialize, Debug)]
struct UpdatePrefix {
    id: i64,
    prefix: Option<String>,
}

#[command]
#[aliases(u)]
#[description("Edit your user settings.")]
#[sub_commands(user_pronoun)]
async fn user(ctx: &Context, msg: &Message) -> CommandResult {
    // Send error message if no subcommands were matched.
    msg.channel_id.say(&ctx.http, "Invalid setting!").await?;

    Ok(())
}

#[command("pronoun")]
#[aliases(pronouns, pn)]
#[description("set/view pronouns.\nex: they/them/their/theirs")]
async fn user_pronoun(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // read from data lock
    let data = ctx.data.read().await;
    // get our db pool from the data lock

    let pool = data.get::<ConnectionPool>().unwrap();

    let username = args.rest();
    match get_members(ctx, msg, username.to_string()).await {
        Ok(u) => {
            let get_pn = sqlx::query_as!(
                UpdatePronoun,
                "
                select id, pronouns
                from users
                where id = $1
                limit 1
                ",
                u.user.id.0 as i64
            )
            .fetch_all(pool)
            .await?;

            let none = &"they/them/their/theirs".to_string();
            let pn = match get_pn.is_empty() {
                true => none,
                false => match &get_pn[0].pronouns {
                    Some(v) => match u.user.id.0 as u64 {
                        756801847699832902 => "t/e/My/p",
                        _ => v,
                    },
                    None => none,
                },
            };
            let pronoun: Vec<&str> = pn.split("/").collect();
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    format!(
                        "It looks like {} pronouns are currently {}.",
                        pronoun[2], pn
                    ),
                )
                .await;
        }
        Err(_) => {
            if username == "" {
                let get_pn = sqlx::query_as!(
                    UpdatePronoun,
                    "
                select id, pronouns
                from users
                where id = $1
                limit 1
                ",
                    msg.author.id.0 as i64
                )
                .fetch_all(pool)
                .await?;
                let none = &"they/them/their/theirs. To change them, run this command but add pronouns to the end.".to_string();
                let pn = match &get_pn[0].pronouns {
                    Some(v) => v,
                    None => none,
                };
                let _ = msg
                    .channel_id
                    .say(&ctx.http, format!("Your pronouns are currently {}", pn))
                    .await;
            } else {
                let re = Regex::new(
                    r"^[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}$",
                )
                .unwrap();
                if !re.is_match(username) {
                    return Err(CommandError::from(
                        "h-This doesn't look like a valid pronoun combination.",
                    ));
                }
                let update_fm = sqlx::query_as!(
                    UpdatePronoun,
                    "
                update users 
                set pronouns = $1
                where id = $2
    
                returning id, pronouns
                ",
                    username,
                    msg.author.id.0 as i64
                )
                .fetch_all(pool)
                .await?;

                if update_fm.is_empty() {
                    let _ = msg
                        .channel_id
                        .say(&ctx.http, "creating a Maki user account...")
                        .await;
                    let update_fm = sqlx::query_as!(
                        UpdatePronoun,
                        "
                insert into users(id, pronouns)
                values($1, $2)
    
                returning id, pronouns
                ",
                        msg.author.id.0 as i64,
                        username
                    )
                    .fetch_all(pool)
                    .await?;
                    let _ = msg
                        .channel_id
                        .say(&ctx.http, format!("{:?}", update_fm))
                        .await;
                } else {
                    let _ = msg
                        .channel_id
                        .say(&ctx.http, format!("{:?}", update_fm))
                        .await;
                }

                let tosay = "".to_string()
                    + &msg.author.tag()
                    + " ("
                    + &msg.author.id.to_string()
                    + ")"
                    + "'s pronouns are saved as "
                    + username;

                let _ = msg.channel_id.say(&ctx.http, tosay).await;
            }
        }
    };
    Ok(())
}

#[command]
#[aliases(sv)]
#[description("Edit the server's settings.")]
#[sub_commands(server_prefix)]
async fn server(ctx: &Context, msg: &Message) -> CommandResult {
    // Send error message if no subcommands were matched.
    msg.channel_id.say(&ctx.http, "Invalid setting!").await?;

    Ok(())
}

#[command("prefix")]
#[aliases(p)]
#[required_permissions(ADMINISTRATOR)]
#[owner_privilege]
async fn server_prefix(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Send error message if no subcommands were matched.
    if let Some(id) = msg.guild_id {
        // read from data lock
        let data = ctx.data.read().await;
        // get our db pool from the data lock
        let pool = data.get::<ConnectionPool>().unwrap();

        let to_set = args.rest();
        if to_set.is_empty() {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    format!(
                        "The prefix for this server is currently {:#?}",
                        dynamic_prefix(ctx, msg).await.unwrap_or(
                            std::env::var("PREFIX").expect("Expected a prefix in the environment.")
                        )
                    ),
                )
                .await;
        } else {
            let prefix = sqlx::query!(
                "
            update guilds 
            set prefix = $1
            where id = $2
    
            returning id, prefix
            ",
                to_set,
                id.0 as i64
            )
            .fetch_all(pool)
            .await?;
            if prefix.is_empty() {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, "creating a Maki user account...")
                    .await;
                let create_prefix = sqlx::query_as!(
                    UpdatePrefix,
                    "
            insert into guilds(id, prefix)
            values($1, $2)
    
            returning id, prefix
            ",
                    id.0 as i64,
                    to_set
                )
                .fetch_all(pool)
                .await?;
                let _ = msg
                    .channel_id
                    .say(&ctx.http, format!("{:?}", create_prefix))
                    .await;
            } else {
                let _ = msg.channel_id.say(&ctx.http, format!("{:?}", prefix)).await;
            }
        }
    } else {
        let _ = msg
            .channel_id
            .say(
                &ctx.http,
                "You are not currently in a server and thus cannot perform this command.",
            )
            .await;
    }

    Ok(())
}
