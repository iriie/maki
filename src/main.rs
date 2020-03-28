extern crate timeago;
extern crate dotenv;
extern crate chrono;
extern crate reqwest;
#[macro_use]
extern crate cached;

use log::{info};
use std::{collections::{HashMap, HashSet}, sync::Arc};
use serenity::{
    framework::standard::{
        Args, CommandResult, CommandGroup,
        DispatchError, HelpOptions, help_commands, StandardFramework,
        macros::{group, help},
    },
    model::{channel::{ Message}, gateway::Ready, id::UserId, event::ResumedEvent},
};
use chrono::Utc;

use dotenv::dotenv;
use std::env;

#[macro_use]
pub mod utils;

mod keys;
use keys::*;

mod commands;
use commands::meta::*;
use commands::general::*;
use commands::music::lastfm::*;
use commands::fun::fun_main::*;
use commands::fun::pokemon::*;

// This imports `typemap`'s `Key` as `TypeMapKey`.
use serenity::prelude::*;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{}#{} is connected!", ready.user.name, ready.user.discriminator);
    }
    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed.");
    }
}

#[group]
#[commands(activity, nickname, quit)]
struct Admin;

#[group]
#[commands(weather, invite, ping, stats)]
struct General;

#[group]
#[commands(urbandictionary, pikachu, pokedex)]
struct Fun;

#[group]
#[commands(lastfm)]
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
fn help(
    ctx: &mut Context,
    msg: &Message,
    args: Args,
    options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(ctx, msg, args, options, groups, owners)
}

fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = &env::var("BOT_TOKEN").expect("Expected a discord token in the environment.");
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    {
        let mut data = client.data.write();
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<SerenityShardManager>(Arc::clone(&client.shard_manager));
        data.insert::<Uptime>(Utc::now());
    }

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };
    client.with_framework(
        // Configures the client, allowing for options to mutate how the
        // framework functions.
        //
        // Refer to the documentation for
        // `serenity::ext::framework::Configuration` for all available
        // configurations.
        StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("~")
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
            .owners(owners))

        // Set a function to be called prior to each command execution. This
        // provides the context of the command, the message that was received,
        // and the full name of the command that will be called.
        //
        // You can not use this to determine whether a command should be
        // executed. Instead, the `#[check]` macro gives you this functionality.
        .before(|ctx, msg, command_name| {
            println!("Got command '{}' by user '{}'",
                     command_name,
                     msg.author.name);

            // Increment the number of times this command has been run once. If
            // the command's name does not exist in the counter, add a default
            // value of 0.
            let mut data = ctx.data.write();
            let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in ShareMap.");
            let entry = counter.entry(command_name.to_string()).or_insert(0);
            *entry += 1;

            true // if `before` returns false, command processing doesn't happen.
        })
        // Similar to `before`, except will be called directly _after_
        // command execution.
        .after(|_, _, command_name, error| {
            match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
            }
        })
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
        .unrecognised_command(|_, _, unknown_command_name| {
            println!("Could not find command named '{}'", unknown_command_name);
        })
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(|ctx, msg, error| {
            if let DispatchError::Ratelimited(seconds) = error {
                let _ = msg.channel_id.say(&ctx.http, &format!("Try this again in {} seconds.", seconds));
            }
        })
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
        .group(&MUSIC_GROUP)
    );

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}