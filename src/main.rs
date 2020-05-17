extern crate pretty_env_logger;
#[macro_use] extern crate log;
use serenity::{
    async_trait,
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
    sync::Arc,
    env,
};

use chrono::Utc;

use dotenv::dotenv;

#[macro_use]
pub mod utils;

pub mod keys;
use keys::*;

mod commands;
use commands::fun::fun_main::*;
use commands::fun::pokemon::*;
use commands::general::*;
use commands::meta::*;
use commands::music::lastfm::*;
use commands::music::spotify::*;
use commands::moderator::*;

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
#[commands(activity, nickname, quit, prune)]
struct Admin;

#[group]
#[commands(weather, invite, ping, stats)]
struct General;

#[group]
#[commands(urbandictionary, pikachu, pokemon)]
struct Fun;

#[group]
#[commands(lastfm, spotify)]
struct Music;

#[help]
#[embed_error_colour(red)]
#[embed_success_colour(fooyoo)]
#[lacking_ownership(hide)]
#[lacking_permissions(hide)]
#[lacking_role(hide)]
#[max_levenshtein_distance(2)]
#[strikethrough_commands_tip_in_dm(false)]
#[strikethrough_commands_tip_in_guild(false)]

async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(ctx, msg, args, options, groups, owners).await
}

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(
    _ctx: &Context,
    msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => warn!(
            "Command `{}` triggered by `{}` has errored: \n{}",
            command_name,
            msg.author.tag(),
            why.0
        ),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, _unknown_command_name: &str) {
    // do nothing, we don't want to annoy people !!!
    //println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, _msg: &Message) {
    // why would anyone enable this, unless they're logging every message???
    //println!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) -> () {
    //for ratelimiting and other things
    if let DispatchError::Ratelimited(seconds) = error {
        let _ = msg
            .channel_id
            .say(
                &ctx.http,
                &format!("Try this again in {} seconds.", seconds),
            )
            .await;
    };
}

// this function should return a prefix as a string
#[hook]
async fn dynamic_prefix(_ctx: &Context, msg: &Message) -> Option<String> {
    // Make sure we can actually get the guild_id, if not there's
    // no point to trying to find the prefix. Also means we can use
    // unwrap for this later on, since we Guard check it's Some() here
    msg.guild_id?;
    let p;

    p = ">".to_string();

    Some(p)
}

#[tokio::main(core_threads = 8)]
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
                .on_mention(Some(bot_id))
                .dynamic_prefix(dynamic_prefix)
                // You can set multiple delimiters via delimiters()
                // or just one via delimiter(",")
                // If you set multiple delimiters, the order you list them
                // decides their priority (from first to last).
                //
                // In this case, if "," would be first, a message would never
                // be delimited at ", ", forcing you to trim your arguments if you
                // want to avoid whitespaces at the start of each.
                .delimiters(vec![", ", ","])
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
        .help(&HELP)
        // Can't be used more than once per 5 seconds:
        //.bucket("emoji", |b| b.delay(5))
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay:
        //.bucket("complicated", |b| b.delay(5).time_span(30).limit(2))
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
        .group(&ADMIN_GROUP)
        .group(&GENERAL_GROUP)
        .group(&FUN_GROUP)
        .group(&MUSIC_GROUP);

        let mut client = Client::new(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<keys::Uptime>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}
