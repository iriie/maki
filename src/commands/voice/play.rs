use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::{
        channel::Message,
        id::{ChannelId, GuildId},
    },
};

use songbird::{CoreEvent, Event, Songbird, TrackEvent};
use std::{sync::Arc, collections::{HashMap, hash_map::RandomState}, ffi::OsStr};

use crate::{utils::queue::TrackQueue, commands::voice::{Receiver, TrackEndNotifier}, keys::VoiceQueue};

async fn get_source<P: AsRef<OsStr>>(
    path: P,
    path_string: &str,
) -> Result<songbird::input::Input, String> {
    dbg!(path_string.split("/").collect::<Vec<&str>>()[2]);
    let source = match path_string.split("/").collect::<Vec<&str>>()[2] {
        _ => match songbird::ffmpeg(&path).await {
            Ok(source) => {
                //source.metadata.title = Some("hi".to_string());
                source
            }
            Err(why) => {
                println!("Err starting source: {:?}", why);

                return Err("fuck".to_string());
            }
        },
    };
    Ok(source)
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    // get our queue
    let data = ctx.data.read().await;
    let qu = data.get::<VoiceQueue>().unwrap();
    let q = qu.write().await;

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // stop playback if going on
    if let Some(queue) = q.get(&guild_id) {
        let _ = queue.stop();
    }

    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            msg.channel_id
                .say(&ctx.http, format!("Failed: {:?}", e))
                .await?;
        }

        msg.channel_id.say(&ctx.http, "Left voice channel").await?;
    } else {
        msg.reply(ctx, "Not in a voice channel").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Must provide a URL to a video or audio")
                .await?;

            return Ok(());
        }
    };

    if !url.starts_with("http") {
        msg.channel_id
            .say(&ctx.http, "Must provide a valid URL")
            .await?;

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // get our queue
    let data = ctx.data.read().await;
    let qu = &mut data.get::<VoiceQueue>().unwrap();
    let mut q = qu.write().await;

    if let Some(handler_lock) = manager.get(guild_id) {
        if let Some(queue) = q.get(&guild_id) {
            let mut handler = handler_lock.lock().await;

            let source = get_source(&url, &url).await?;

            //handler.enqueue_source(source);
            queue.add_source(source, &mut handler);

            msg.channel_id
                .say(
                    &ctx.http,
                    format!(
                        "Added song to queue: position {}",
                        queue.len() - 1
                    ),
                )
                .await?;
        }
    } else {
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                msg.reply(ctx, "Not in a voice channel").await?;

                return Ok(());
            }
        };

        connect_and_register(
            manager.clone(),
            ctx,
            msg,
            guild_id,
            connect_to,
            msg.channel_id,
            qu,
        )
        .await?;

        q.insert(guild_id, TrackQueue::new());

        if let Some(handler_lock) = manager.get(guild_id) {
            if let Some(queue) = q.get(&guild_id) {
                let mut handler = handler_lock.lock().await;

                let source = get_source(&url, &url).await?;

                //handler.enqueue_source(source);
                queue.add_source(source, &mut handler);

                dbg!(queue);

                match queue.len() {
                    1 => {
                        msg.channel_id
                            .say(&ctx.http, format!("Now playing a song"))
                            .await?
                    }
                    _ => {
                        msg.channel_id
                            .say(
                                &ctx.http,
                                format!(
                                    "Added song to queue: position {}",
                                    q.len() - 1
                                ),
                            )
                            .await?
                    }
                };
            } else {println!("no guild id found in queue")}
        } else {println!("no guild id found in handler")}
    }

    Ok(())
}

async fn connect_and_register(
    manager: std::sync::Arc<Songbird>,
    ctx: &Context,
    msg: &Message,
    guild_id: GuildId,
    voice: ChannelId,
    text: ChannelId,
    queue: &mut &Arc<RwLock<HashMap<GuildId, TrackQueue, RandomState>>>,
) -> Result<(), CommandError> {
    let (handler_lock, conn_result) = manager.join(guild_id, voice).await;

    if let Ok(_) = conn_result {
        // NOTE: this skips listening for the actual connection result.
        let mut handler = handler_lock.lock().await;

        let send_http = ctx.http.clone();

        handler.add_global_event(
            Event::Track(TrackEvent::End),
            TrackEndNotifier {
                guild_id,
                chan_id: text,
                http: send_http,
                //manager,
                queue: queue.to_owned()
            },
        );

        handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), Receiver::new());

        handler.add_global_event(CoreEvent::SpeakingUpdate.into(), Receiver::new());

        handler.add_global_event(CoreEvent::VoicePacket.into(), Receiver::new());

        handler.add_global_event(CoreEvent::RtcpPacket.into(), Receiver::new());

        handler.add_global_event(CoreEvent::ClientConnect.into(), Receiver::new());

        handler.add_global_event(CoreEvent::ClientDisconnect.into(), Receiver::new());

        msg.channel_id
            .say(
                &ctx.http,
                &format!(
                    "Joined 🔊 {}, bound to #️⃣ {}",
                    voice.mention(),
                    text.mention()
                ),
            )
            .await?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Error joining the channel")
            .await?;
    };
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn queue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.is_empty() {
        false => play(ctx, msg, args).await?,
        true => {
            msg.channel_id
                .say(&ctx.http, "TODO: Queue function")
                .await?;

            return Ok(());
        }
    };

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {

        // get our queue
        let data = ctx.data.read().await;
        let qu = &mut data.get::<VoiceQueue>().unwrap();
        let q = qu.write().await;

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if let Some(queue) = q.get(&guild_id) {
        let _ = queue.skip();

        msg.channel_id
            .say(
                &ctx.http,
                format!("{} song skipped in queue.", queue.len()),
            )
            .await?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel to play in")
            .await?;
    }

    Ok(())
}
