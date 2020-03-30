use crate::SerenityShardManager;
use crate::Uptime;
use log::{error};
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use chrono::DateTime;
use chrono::Utc;
use timeago;

struct Timer {
    start: DateTime<Utc>,
}

impl Timer {
    pub fn new() -> Self {
        Timer { start: Utc::now() }
    }

    pub fn elapsed_ms(&self) -> i64 {
        Utc::now()
            .signed_duration_since(self.start)
            .num_milliseconds()
    }
}

#[command]
#[aliases(presence, a)]
#[description("Edit the bot's presence. Use the `listen`, `play`, or `reset` subcommands to set the respective activity.")]
#[owners_only]
#[sub_commands(activity_listen, activity_play, activity_stream, activity_reset)]
fn activity(ctx: &mut Context, msg: &Message) -> CommandResult {
    // Send error message if no subcommands were matched.
    msg.channel_id.say(&ctx.http, "Invalid activity!")?;

    Ok(())
}

#[command("listen")]
fn activity_listen(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let activity = Activity::listening(args.rest());
    ctx.set_activity(activity);

    msg.channel_id
        .say(&ctx.http, format!("Now listening to `{:#?}`", args.rest()))?;

    Ok(())
}

#[command("play")]
fn activity_play(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let activity = Activity::playing(args.rest());
    ctx.set_activity(activity);

    msg.channel_id
    .say(&ctx.http, format!("Now playing `{:#?}`", args.rest()))?;

    Ok(())
}

#[command("stream")]
fn activity_stream(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let stream_url: &str = "https://twitch.tv/smallant1";
    let activity = Activity::streaming(args.rest(), stream_url);
    ctx.set_activity(activity);

    msg.channel_id
    .say(&ctx.http, format!("Now streaming `{:#?}`", args.rest()))?;

    Ok(())
}

#[command("reset")]
fn activity_reset(ctx: &mut Context, msg: &Message) -> CommandResult {
    ctx.reset_presence();

    msg.channel_id
        .say(&ctx.http, "Reset presence successully!")?;

    Ok(())
}

#[command]
#[description("Invite the bot to a server.")]
fn invite(ctx: &mut Context, msg: &Message) -> CommandResult {
    // Create invite URL using the bot's user ID.
    let url = format!("Invite URL: <https://discordapp.com/oauth2/authorize?&client_id={}&scope=bot&permissions=0>", ctx.cache.read().user.id);

    msg.channel_id.say(&ctx.http, url)?;

    Ok(())
}

#[command]
#[aliases(nick)]
#[description("Edit the bot's nickname on a server. Pass no arguments to reset nickname.")]
#[only_in(guilds)]
#[owners_only]
fn nickname(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if let Some(guild) = msg.guild_id {
        // Reset nickname if no args given.
        let name = if args.is_empty() {
            None
        } else {
            Some(args.message())
        };

        if let Err(why) = guild.edit_nickname(&ctx.http, name) {
            error!("Error changing nickname: {:?}", why);
        }
            let fmt = format!("Changed nickname to `{:#?}`", args.message());
            let _ = match msg.channel_id.say(&ctx.http, fmt){
                Ok(_) => return Ok(()),
                Err(_) => return Ok(()),
            };
    }

    Ok(())
}

#[command]
#[description("Pings Discord and shows ping time.")]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    let timer = Timer::new();

    let mut sent_msg = match msg.channel_id.say(&ctx.http, "Ping!") {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };

    let msg_ms = timer.elapsed_ms();

    let runner_latency = {
        let data = ctx.data.read();
        let shard_manager = match data.get::<SerenityShardManager>() {
            Some(v) => v,
            None => {
                return Err(CommandError::from(
                    "There was a problem getting the shard manager",
                ))
            }
        };

        let manager = shard_manager.lock();
        let runners = manager.runners.lock();

        // Shards are backed by a "shard runner" responsible for processing events
        // over the shard, so we'll get the information about the shard runner for
        // the shard this command was sent over.
        let runner = match runners.get(&ShardId(ctx.shard_id)) {
            Some(runner) => runner,
            None => return Err(CommandError::from("No shard found")),
        };

        runner.latency
    };

    let runner_latency_ms = runner_latency.map(|x| {
        format!(
            "{:.3}",
            x.as_secs() as f64 / 1000.0 + f64::from(x.subsec_nanos()) * 1e-6
        )
    });

    let _ = sent_msg.edit(&ctx, |m| {
        m.content(&format!(
            "API latency: `{} ms`\n\
                Shard latency: `{} ms`\n",
            msg_ms,
            runner_latency_ms.unwrap_or("(shard not found)".into()),
        ))
    });
    Ok(())
}

#[command]
#[description("Bot stats")]
fn stats(ctx: &mut Context, msg: &Message) -> CommandResult {

    let bot_version = env!("CARGO_PKG_VERSION");
    let build_number = option_env!("BUILD_BUILDNUMBER");
    let agent_name = option_env!("AGENT_MACHINENAME");
    let agent_id = option_env!("AGENT_ID");

    let (name, discriminator) = match ctx.http.get_current_application_info() {
        Ok(info) => {
            (info.owner.name, info.owner.discriminator)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let owner_tag = name.to_string() + "#" + &discriminator.to_string();

    let guilds_count = ctx.cache.read().guilds.len();
    let channels_count = ctx.cache.read().channels.len();
    let users_count = ctx.cache
        .read()
        .guilds
        .values()
        .fold(0, |acc, x| acc + x.read().member_count);
    let users_count_unique = ctx.cache.read().users.len();

    let current_time = Utc::now();
    let start_time = {
        let data = ctx.data.read();
        match data.get::<Uptime>() {
            Some(val) => *val,
            None => {
                return Err(CommandError::from(
                    "There was a problem getting the shard manager",
                ))
            }
        }
    };

    let mut f = timeago::Formatter::new();
    f.num_items(4);
    f.ago("");

    let uptime_humanized = f.convert_chrono(start_time, current_time);

    let _ = msg.channel_id.send_message(&ctx.http,|m|
        m.embed(|e| e
            .color(0x3498db)
            .title(&format!(
                "maki v{} - build #{} ({} #{})",
                bot_version, build_number.unwrap_or("N/A"),
                agent_name.unwrap_or("N/A"), agent_id.unwrap_or("N/A")
            ))
            .url("https://maki.kanbaru.me")
            .field("Author", &owner_tag, true)
            .field("Guilds", &guilds_count.to_string(), true)
            .field("Channels", &channels_count.to_string(), true)
            .field("Users", &format!("{} Total\n{} Unique (cached)",
                users_count, users_count_unique), true)
            .field("Bot Uptime", &uptime_humanized, false)
        )
    );

    Ok(())
}

#[command]
#[aliases(shutdown)]
#[description("Shut down the bot.")]
#[owners_only]
fn quit(ctx: &mut Context, msg: &Message) -> CommandResult {
    use crate::SerenityShardManager;

    let data = ctx.data.read();

    if let Some(manager) = data.get::<SerenityShardManager>() {
        msg.channel_id.say(&ctx.http, "Shutting down!")?;

        // Shut down all shards.
        manager.lock().shutdown_all();
    } else {
        error!("There was a problem getting the shard manager.");
    }

    Ok(())
}
