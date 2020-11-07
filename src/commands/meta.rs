use crate::keys::ShardManagerContainer;
use crate::Uptime;
use chrono::DateTime;
use chrono::Utc;
use heim::{memory, process, units};
use log::error;
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use timeago;
use tokio::time;

use tokio::process::Command;

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
#[sub_commands(activity_listen, activity_play, activity_stream, activity_compete, activity_reset)]
async fn activity(ctx: &Context, msg: &Message) -> CommandResult {
    // Send error message if no subcommands were matched.
    msg.channel_id.say(&ctx.http, "Invalid activity!").await?;

    Ok(())
}

#[command("listen")]
async fn activity_listen(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let activity = Activity::listening(args.rest());
    ctx.set_activity(activity).await;

    msg.channel_id
        .say(&ctx.http, format!("Now listening to `{:#?}`", args.rest()))
        .await?;

    Ok(())
}

#[command("play")]
async fn activity_play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let activity = Activity::playing(args.rest());
    ctx.set_activity(activity).await;

    msg.channel_id
        .say(&ctx.http, format!("Now playing `{:#?}`", args.rest()))
        .await?;

    Ok(())
}

#[command("stream")]
async fn activity_stream(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let stream_url: &str = "https://twitch.tv/smallant1";
    // random streamer i like i guess^^^^?
    let activity = Activity::streaming(args.rest(), stream_url);
    ctx.set_activity(activity).await;

    msg.channel_id
        .say(&ctx.http, format!("Now streaming `{:#?}`", args.rest()))
        .await?;

    Ok(())
}

#[command("compete")]
async fn activity_compete(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let activity = Activity::competing(args.rest());
    ctx.set_activity(activity).await;

    msg.channel_id
        .say(&ctx.http, format!("Now competing in `{:#?}`", args.rest()))
        .await?;

    Ok(())
}

#[command("reset")]
async fn activity_reset(ctx: &Context, msg: &Message) -> CommandResult {
    ctx.reset_presence().await;

    msg.channel_id
        .say(&ctx.http, "Reset presence successully!")
        .await?;

    Ok(())
}

#[command]
#[description("Invite the bot to a server.")]
async fn invite(ctx: &Context, msg: &Message) -> CommandResult {
    // Create invite URL using the bot's user ID.
    let url = format!("Invite URL: <https://discordapp.com/oauth2/authorize?&client_id={:?}&scope=bot&permissions=0>", ctx.cache.current_user_id().await);

    msg.channel_id.say(&ctx.http, url).await?;

    Ok(())
}

#[command]
#[aliases(nick)]
#[description("Edit the bot's nickname on a server. Pass no arguments to reset nickname.")]
#[only_in(guilds)]
#[owners_only]
async fn nickname(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if let Some(guild) = msg.guild_id {
        // Reset nickname if no args given.
        let name = if args.is_empty() {
            None
        } else {
            Some(args.message())
        };

        if let Err(why) = guild.edit_nickname(&ctx.http, name).await {
            error!("Error changing nickname: {:?}", why);
        }
        let fmt = format!("Changed nickname to `{:#?}`", args.message());
        let _ = match msg.channel_id.say(&ctx.http, fmt).await {
            Ok(_) => return Ok(()),
            Err(_) => return Ok(()),
        };
    }

    Ok(())
}

#[command]
#[description("Pings Discord and shows ping time.")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let timer = Timer::new();

    let sent_msg = match msg.channel_id.say(&ctx.http, "Ping!").await {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };
    let msg_ms = timer.elapsed_ms();

    let data = ctx.data.read().await;
    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    let runner_latency_ms = runner.latency.map(|x| {
        format!(
            "{:.3}",
            x.as_secs() as f64 / 1000.0 + f64::from(x.subsec_nanos()) * 1e-6
        )
    });

    let _ = sent_msg
        .clone()
        .edit(ctx, |m| {
            m.content(&format!(
                "Pong! \n\
            API latency: `{} ms`\n\
            Shard latency: `{} ms`\n",
                msg_ms,
                runner_latency_ms
                    .clone()
                    .unwrap_or("(shard not found)".into()),
            ))
        })
        .await?;

    Ok(())
}

#[command]
#[description("Bot stats")]
async fn stats(ctx: &Context, msg: &Message) -> CommandResult {

    let bot_version = env!("CARGO_PKG_VERSION");

    let memory = memory::memory().await.unwrap();
    // get current process
    let process = process::current().await.unwrap();
    // get current ram
    let thismem = process.memory().await.unwrap();
    let fullmem = memory.total();
    // get current cpu
    let cpu_1 = process.cpu_usage().await.unwrap();

    time::delay_for(time::Duration::from_millis(100)).await;

    let cpu_2 = process.cpu_usage().await.unwrap();

    let git_stdout;
    git_stdout = Command::new("sh")
        .arg("-c")
        .arg("git log -1 | grep ^commit | awk '{print $2}'")
        .output()
        .await.unwrap();

    let mut git_commit: String = "".to_string();

    if std::str::from_utf8(&git_stdout.stdout).unwrap() != "" {
        git_commit.push('#');
        git_commit.push_str(std::str::from_utf8(&git_stdout.stdout).unwrap());
    } else {
        git_commit.push_str("prod")
    }
    git_commit.truncate(7);


    let (name, discriminator) = match ctx.http.get_current_application_info().await {
        Ok(info) => (info.owner.name, info.owner.discriminator),
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let owner_tag = name.to_string() + "#" + &discriminator.to_string();

    let guilds_count = &ctx.cache.guilds().await.len();
    let channels_count = &ctx.cache.guild_channel_count().await;
    let users_count = ctx.cache.user_count().await;
    let users_count_unknown = ctx.cache.unknown_members().await as usize;

    let uptime = {
        let data = ctx.data.read().await;
        match data.get::<Uptime>() {
            Some(time) => {
                if let Some(boot_time) = time.get("boot") {
                    let now = Utc::now();
                    let mut f = timeago::Formatter::new();
                    f.num_items(4);
                    f.ago("");

                    f.convert_chrono(boot_time.to_owned(), now)
                } else {
                    "Uptime not available".to_owned()
                }
            }
            None => "Uptime not available.".to_owned(),
        }
    };

    let mut f = timeago::Formatter::new();
    f.num_items(4);
    f.ago("");

    let shard_plural = if ctx.cache.shard_count().await > 1 { "s" } else { "" };
    let avatar = ctx.cache.current_user().await.avatar_url().unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_string());
    let shards = ctx.cache.shard_count().await;

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(0x3498db)
                    .title(&format!("maki v{} {}", bot_version, git_commit,))
                    .url("https://maki.iscute.dev")
                    .thumbnail(&format!("{}", avatar))
                    .field("Author", &owner_tag, false)
                    .field("Guilds", &guilds_count.to_string(), true)
                    .field("Channels", &channels_count.to_string(), true)
                    .field(
                        "Users",
                        &format!(
                            "`{} Total`\n`{} Cached`",
                            &users_count + &users_count_unknown,
                            users_count
                        ),
                        true,
                    )
                    .field(
                        "Memory",
                        format!(
                            "`{} MB used`\n`{} MB virt`\n`{} GB available`",
                            &thismem.rss().get::<units::information::megabyte>(),
                            &thismem.vms().get::<units::information::megabyte>(),
                            &fullmem.get::<units::information::gigabyte>()
                        ),
                        true,
                    )
                    .field(
                        "CPU",
                        format!("`{}%`", (cpu_2 - cpu_1).get::<units::ratio::percent>()),
                        true,
                    )
                    .field(
                        "Shards",
                        format!("`{} shard{}` ", shards, shard_plural),
                        true,
                    )
                    .field("Bot Uptime", &uptime, false);
                e
            });
            m
        })
        .await;

    Ok(())
}

#[command]
#[aliases(shutdown, kill)]
#[description("Shut down the bot.")]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.channel_id.say(&ctx.http, "Shutting down!").await?;

        // Shut down all shards.
        manager.lock().await.shutdown_all().await;
    } else {
        error!("There was a problem getting the shard manager.");
    }

    Ok(())
}
