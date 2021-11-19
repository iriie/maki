extern crate pretty_env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use songbird::SerenityInit;

use serenity::{
    async_trait,
    client::bridge::gateway::GatewayIntents,
    framework::standard::{
        help_commands,
        macros::{group, help, hook},
        Args, CommandGroup, CommandResult, DispatchError, HelpOptions, StandardFramework,
    },
    http::Http,
    model::{channel::Message, event::ResumedEvent, gateway::Ready, id::UserId, prelude::GuildId},
};
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
};

use chrono::Utc;

use dotenv::dotenv;

use rand::random;

#[macro_use]
pub mod utils;

pub mod keys;
use keys::*;

mod commands;
use commands::fun::fun_main::*;
use commands::fun::pokemon::*;
use commands::general::*;
use commands::meta::*;
use commands::moderator::*;
use commands::music::lastfm::*;
use commands::music::spotify::*;
use commands::settings::*;
use commands::voice::play::*;

use utils::db::get_pool;

// This imports `typemap`'s `Key` as `TypeMapKey`.
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, _ctx: Context, guilds: Vec<GuildId>) {
        info!("Connected to {} guilds.", guilds.len());
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            info!(
                "Connected as {} on shard {}/{}",
                ready.user.name,
                shard[0] + 1,
                shard[1]
            );
        } else {
            info!("Connected as {}", ready.user.name);
        }

        // puts current time (startup) in uptime key, to be used later
        let data = ctx.data.write();
        match data.await.get_mut::<Uptime>() {
            Some(uptime) => {
                uptime.entry(String::from("boot")).or_insert_with(Utc::now);
            }
            None => error!("Unable to insert boot time into client data."),
        };
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(activity, nickname, quit, shorten)]
#[description = "admin/bot management stuff."]
struct Admin;

#[group]
#[commands(prune, kick, ban, mute)]
#[description = "server management stuff."]
struct Moderation;

#[group]
#[commands(translate, weather, invite, ping, stats, say, urbandictionary)]
#[description = "general stuff, or stuff that won't fit anywhere else."]
struct General;

#[group]
#[commands(ship, pikachu, pokemon)]
#[description = "funny gaming."]
struct Fun;

#[group]
#[commands(hug, pat)]
#[description = "gets gifs from other services (lolol)"]
struct Gif;

#[group]
#[commands(lastfm, spotify)]
#[description = "search or show your own music."]
struct Music;

#[group]
#[commands(join, play, skip, queue, leave)]
#[description = "play music in a voice channel."]
struct Voice;

#[group]
#[commands(server, user)]
#[description = "settings for different things implemented into the bot"]
struct Settings;

#[help]
#[individual_command_tip = "for more info about a command or group, pass the name as a subcommand."]
#[lacking_ownership = "Hide"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Hide"]
#[max_levenshtein_distance(2)]
#[strikethrough_commands_tip_in_dm = "you cannot run ~~strikethroughed commands~~."]
#[strikethrough_commands_tip_in_guild = "you cannot run ~~strikethroughed commands~~."]
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let ho = help_options.clone();
    let _ = help_commands::with_embeds(ctx, msg, args, &ho, groups, owners).await;
    Ok(())
}

fn rand_str(length: u32) -> String {
    (0..length)
        .map(|_| (0x2du8 + (random::<f32>() * 79.0) as u8) as char)
        .collect()
}

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    debug!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: CommandResult) {
    if let Err(why) = &error {
        if !&format!("{:?}", why).contains("h-") {
            let error_code = rand_str(7).replace("`", ",");
            error!(
                "Error while running command {} (code {})",
                &cmd_name, error_code
            );
            error!("{:?}", &error);
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!(
                        "Something went wrong!\nerror: `{}` | id: `{}`",
                        format!("{:?}", why),
                        error_code
                    ),
                )
                .await;
        } else {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!(
                        "Something went wrong!\nerror: `{}`",
                        format!("{:?}", why).replace("h-", "")
                    ),
                )
                .await;
        }
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    // do nothing, we don't want to annoy people !!!
    debug!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    debug!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn prefix_only(ctx: &Context, msg: &Message) {
    let prefix = env::var("PREFIX").expect("Expected a prefix in the environment.");
    if msg.content == "<@!683934524526034994>".to_string()
        || msg.content == "<@683934524526034994>".to_string()
    {
        let prefix = dynamic_prefix(ctx, msg).await.unwrap_or(prefix);
        const is_prod: &str =
            match &env::var("PRODUCTION").expect("Expected a prefix in the environment.").to_lower() == "true" {
                true => "",
                false => " beta",
            };
        let _ = msg
            .channel_id
            .say(&ctx.http, &format!("The{} prefix is `{}`", is_prod, prefix))
            .await;
    }
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _cmd_name: &str) -> () {
    //for ratelimiting and other things
    match error {
        DispatchError::Ratelimited(seconds) => {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {:?} seconds.", seconds),
                )
                .await;
        }
        DispatchError::NotEnoughArguments { min, given } => {
            let ret = {
                if given > 1 {
                    format!("{} arguments needed but {} were provided.", min, given)
                } else if given == 1 {
                    format!("{} arguments needed but {} was provided.", min, given)
                } else {
                    format!("{} arguments needed.", min)
                }
            };
            let _ = msg.channel_id.say(&ctx.http, ret).await;
        }
        _ => {
            error!("Dispatch error: {:?}", error);
        }
    }
}

// this function should return a prefix as a string
#[hook]
pub async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
    // get the default prefix
    let token = &env::var("PREFIX").expect("Expected a prefix in the environment.");
    let is_prod = &env::var("PRODUCTION").expect("Expected a prefix in the environment.");

    let p;
    // if sent from a guild, we check for a prefix in the database
    // TODO: find some way to cache this
    if let Some(id) = msg.guild_id {
        if is_prod != &"true".to_string() {
            return Some(token.to_string());
        }
        // read from data lock
        let data = ctx.data.read().await;
        // get our db pool from the data lock
        let pool = data.get::<ConnectionPool>().unwrap();

        let prefix = sqlx::query!(
            "
        select id, prefix
        from guilds
        where id = $1
        limit 1
        ",
            id.0 as i64
        )
        .fetch_optional(pool)
        .await
        .expect("Could not query the database");

        p = if let Some(result) = prefix {
            result.prefix.unwrap_or(".".to_string()).to_string()
        } else {
            token.to_string()
        };
    } else {
        p = token.to_string()
    }
    Some(p)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    // Configure the client with your Discord bot token in the environment.
    let token = &env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment.");
    // Note: We create the client a bit further down

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };
    // Configures the client, allowing for options to mutate how the
    // framework functions.
    //
    // Refer to the documentation for
    // `serenity::ext::framework::Configuration` for all available
    // configurations.
    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(UserId(bot_id.as_u64().to_owned())))
                .dynamic_prefix(dynamic_prefix)
                .prefix("bahahaha")
                // You can set multiple delimiters via delimiters()
                // or just one via delimiter(",")
                // If you set multiple delimiters, the order you list them
                // decides their priority (from first to last).
                //
                // In this case, if "," would be first, a message would never
                // be delimited at ", ", forcing you to trim your arguments if you
                // want to avoid whitespaces at the start of each.
                .delimiters(vec![", ", ",", " "])
                // Sets the bot's owners. These will be used for commands that
                // are owners only.
                .owners(owners)
        })
        // Set a function to be called prior to each command execution. This
        // provides the context of the command, the message that was received,
        // and the full name of the command that will be called.
        //
        // You can not use this to determine whether a command should be
        // executed. Instead, the `#[check]` macro gives you this functionality.
        //
        // **Note**: Async closures are unstable, you may use them in your
        // application if you are fine using nightly Rust.
        // If not, we need to provide the function identifiers to the
        // hook-functions (before, after, normal, ...).
        .prefix_only(prefix_only)
        .before(before)
        // Similar to `before`, except will be called directly _after_
        // command execution.
        .after(after)
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
        .unrecognised_command(unknown_command)
        // Set a function that's called whenever a message is not a command.
        .normal_message(normal_message)
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(dispatch_error)
        .help(&MY_HELP)
        // Can't be used more than once per 5 seconds:
        //.bucket("emoji", |b| b.delay(5))
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay:
        //.bucket("complicated", |b| b.delay(5).time_span(30).limit(2))
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
        .group(&ADMIN_GROUP)
        .group(&MODERATION_GROUP)
        .group(&GENERAL_GROUP)
        .group(&FUN_GROUP)
        .group(&GIF_GROUP)
        .group(&MUSIC_GROUP)
        .group(&VOICE_GROUP)
        .group(&SETTINGS_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .intents({
            let mut intents = GatewayIntents::all();
            intents.remove(GatewayIntents::GUILD_PRESENCES);
            intents.remove(GatewayIntents::DIRECT_MESSAGE_TYPING);
            intents.remove(GatewayIntents::GUILD_MESSAGE_TYPING);
            intents
        })
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<keys::Uptime>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        let pool = get_pool().await.unwrap();
        data.insert::<ConnectionPool>(pool.clone());
        data.insert::<VoiceQueue>(Arc::new(tokio::sync::RwLock::new(HashMap::default())));
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}
