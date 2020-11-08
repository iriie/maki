use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use regex::Regex;

use serde;
use serde::{Deserialize, Serialize};

use crate::dynamic_prefix;
use crate::keys::ConnectionPool;
use crate::utils::user::{get_members, get_pronouns};
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

#[derive(Deserialize, Serialize, Debug)]
struct User {
    id: i64,
    pronouns: Option<String>,
    lastfm: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct UpdateLastFM {
    id: i64,
    lastfm: Option<String>,
}

#[command]
#[aliases(u)]
#[description("Edit your user settings.")]
#[sub_commands(user_pronoun, user_lastfm)]
async fn user(ctx: &Context, msg: &Message) -> CommandResult {
    // Send error message if no subcommands were matched.
    msg.channel_id.say(&ctx.http, "Invalid setting!").await?;

    Ok(())
}

#[command("pronoun")]
#[aliases(pronouns, pn)]
#[description("set/view pronouns.\npronoun ex: they/them/their/theirs")]
async fn user_pronoun(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // read from data lock
    let data = ctx.data.read().await;
    // get our db pool from the data lock

    let pool = data.get::<ConnectionPool>().unwrap();

    let username = args.rest();
    if username == "help" {
        let _ = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.color(0x3498db)
                    .description(format!(
                        "A pronoun, in this case, refers to [english personal pronouns](https://en.wikipedia.org/wiki/English_personal_pronouns).
It is formatted like `subject/object/dep. possessive/indep. possessive`, all in the singular case.
For example, usually gender-neutral pronouns would be `they/them/their/theirs` and feminine pronouns would be `she/her/her/hers`.
If you see something wrong, feel free to join [the support server](https://r.izu.moe/discord) and voice your opinions"
                    ))
                })
            })
            .await;
        return Ok(());
    }
    match get_members(ctx, msg, username.to_string()).await {
        Ok(u) => {
            let pn = get_pronouns(u.user, ctx).await;
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
                let pn = if &get_pn.len() >= &1 {
                    match &get_pn[0].pronouns {
                        Some(v) => v,
                        None => none,
                    }
                } else {
                    none
                };
                let _ = msg
                    .channel_id
                    .say(&ctx.http, format!("Your pronouns are currently {}", pn))
                    .await;
            } else {
                if !does_pn_match(username) {
                    return Err(CommandError::from(
                        "h-This doesn't look like a valid pronoun combination. I use this regex to check pronouns:` `/^[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}$/g`\n`For more info do @maki user pn help.",
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
                    let _ = sqlx::query_as!(
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

#[command("lastfm_username")]
#[aliases(fmuser)]
#[description("Change your last.fm username here.")]
pub async fn user_lastfm(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // read from data lock
    let data = ctx.data.read().await;
    // get our db pool from the data lock

    let pool = data.get::<ConnectionPool>().unwrap();

    let username = args.rest();
    match get_members(ctx, msg, username.to_string()).await {
        Ok(u) => {
            let pn = get_pronouns(u.user, ctx).await;
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
            let update_fm = sqlx::query_as!(
                UpdateLastFM,
                "
            update users 
            set lastfm = $1
            where id = $2

            returning id, lastfm
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
                let _ = sqlx::query_as!(
                    UpdateLastFM,
                    "
            insert into users(id, lastfm)
            values($1, $2)

            returning id, lastfm
            ",
                    msg.author.id.0 as i64,
                    username
                )
                .fetch_all(pool)
                .await?;
            }

            let tosay = "".to_string()
                + &msg.author.tag()
                + " ("
                + &msg.author.id.to_string()
                + ")"
                + "'s last.fm username is saved as "
                + username;

            let _ = msg.channel_id.say(&ctx.http, tosay).await;
        }
    }
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
                    .say(&ctx.http, "creating a Maki server profile...")
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
fn does_pn_match(text: &str) -> bool {
    lazy_static! {
            //r"^[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}$",
    static ref RE: Regex = Regex::new(r"^[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}/[a-z0-9_-]{1,6}$").unwrap();
    }
    RE.is_match(text)
}
