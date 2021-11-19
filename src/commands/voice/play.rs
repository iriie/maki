use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::{
        channel::Message,
        id::{ChannelId, GuildId},
    },
};

use songbird::{CoreEvent, Event, Songbird, TrackEvent};
use std::{
    collections::{hash_map::RandomState, HashMap},
    ffi::OsStr,
    sync::Arc,
};

use crate::{
    commands::voice::{Receiver, TrackEndNotifier, WHITELISTED_GUILDS_CHECK},
    keys::VoiceQueue,
    utils::queue::TrackQueue,
};

async fn get_source<P: AsRef<OsStr>>(
    path: P,
    path_string: &str,
) -> Result<songbird::input::Input, String> {
    dbg!(path_string.split("/").collect::<Vec<&str>>()[2]);
    let source = match path_string.split("/").collect::<Vec<&str>>()[2] {
        "audius.co" => {
            let testpath_dash = path_string.clone().split("-").collect::<Vec<_>>();
            let id = testpath_dash[testpath_dash.len() - 1];

            let testpath_slash = path_string.clone().split("/").collect::<Vec<_>>();
            let slug = testpath_slash[testpath_slash.len() - 1]
                .replace(&("-".to_owned() + &id.to_string()), "");
            let artist = testpath_slash[testpath_slash.len() - 2];

            let url = format!(
                "https://creatornode--linustek.repl.co/api/generate.m3u8?id={}&title={}&handle={}",
                id, slug, artist
            );
            dbg!(&url);

            match songbird::ffmpeg(url).await {
                Ok(source) => {
                    //source.metadata.title = Some("hi".to_string());
                    source
                }
                Err(why) => {
                    println!("Err starting source: {:?}", why);

                    return Err("fuck. a ffmpeg error.".to_string());
                }
            }
        }
        "www.youtube.com" | "youtube.com" | "youtu.be" | "soundcloud.com" => {
            match songbird::ytdl(&path_string).await {
                Ok(source) => {
                    info!("youtube-dl track added");
                    source
                }
                Err(why) => {
                    println!("Err starting source: {:?}", why);

                    return Err("fuck. a youtube-dl error.".to_string());
                }
            }
        }
        _ => match songbird::ffmpeg(&path).await {
            Ok(source) => {
                //source.metadata.title = Some("hi".to_string());
                source
            }
            Err(why) => {
                println!("Err starting source: {:?}", why);

                return Err("fuck. a ffmpeg error.".to_string());
            }
        },
    };
    Ok(source)
}

#[command]
#[only_in(guilds)]
#[checks(whitelisted_guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    // get our queue
    let data = ctx.data.read().await;
    let qu = data.get::<VoiceQueue>().unwrap();
    let q = qu.write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
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

        msg.channel_id.say(&ctx.http, "👋 Bye! See you again soon!").await?;
    } else {
        msg.reply(ctx, "Where do you want me to leave? You should be in a voice channel to execute this command.").await?;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(whitelisted_guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "You need to provide a URL to a video or audio")
                .await?;

            return Ok(());
        }
    };

    if !url.starts_with("http") {
        msg.channel_id
            .say(&ctx.http, "You need to provide a valid URL")
            .await?;

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    debug!("Got manager");

    // get our queue
    let data = ctx.data.read().await;
    let qu = &mut data.get::<VoiceQueue>().unwrap();
    let mut q = qu.write().await;

    debug!("Got queue");

    if let Some(handler_lock) = manager.get(guild_id) {
        if let Some(queue) = q.get(&guild_id) {
            let mut handler = handler_lock.lock().await;

            debug!("Got audio handler");

            let source = get_source(&url, &url).await?;
            let metadata = source.metadata.clone();

            //handler.enqueue_source(source);
            queue.add_source(source, &mut handler);

            match queue.len() {
                1 => {
                    let none_value = "Unknown".to_string();
                    let title = match metadata.title {
                        Some(t) => t,
                        None => match metadata.track {
                            Some(t) => t,
                            None => none_value,
                        },
                    };
                    msg.channel_id
                        .say(
                            &ctx.http,
                            format!(
                                "Now playing: `{}` by `{}`",
                                title,
                                metadata.artist.unwrap_or("unknown".to_string())
                            ),
                        )
                        .await?;
                }
                _ => {
                    let none_value = "song".to_string();
                    let title = match metadata.title {
                        Some(t) => t,
                        None => match metadata.track {
                            Some(t) => t,
                            None => none_value,
                        },
                    };
                    let vari = match queue.len() - 1 {
                        1 => "It's up next.".to_string(),
                        2 => "It'll play after the next track.".to_string(),
                        _ => format!("It'll play after the next {} tracks.",  queue.len() - 2),
                    };
                    msg.channel_id
                        .say(
                            &ctx.http,
                            format!(
                                "Added `{}` to queue. {}",
                                title,
                                vari
                            ),
                        )
                        .await?;
                }
            }
        }
    } else {
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                msg.reply(ctx, "Where do you want me to join? You need to be in a voice channel for this.").await?;

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
                let metadata = source.metadata.clone();

                //handler.enqueue_source(source);
                queue.add_source(source, &mut handler);

                dbg!(queue);

                let none_value = "Unknown".to_string();
                let title = match metadata.title {
                    Some(t) => t,
                    None => match metadata.track {
                        Some(t) => t,
                        None => none_value,
                    },
                };

                match queue.len() {
                    1 => {
                        msg.channel_id
                            .say(
                                &ctx.http,
                                format!(
                                    "Now playing: `{}` by `{}`",
                                    title,
                                    metadata.artist.unwrap_or("unknown".to_string())
                                ),
                            )
                            .await?;
                    }
                    _ => {
                        msg.channel_id
                            .say(
                                &ctx.http,
                                format!(
                                    "Added `{}` to queue at position {}",
                                    title,
                                    queue.len() - 1
                                ),
                            )
                            .await?;
                    }
                }
            } else {
                println!("no guild id found in queue")
            }
        } else {
            println!("no guild id found in handler")
        }
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
                queue: queue.to_owned(),
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
            .say(&ctx.http, "I couldn't join the channel. Make sure (or get someone to make sure) that I have permissions to join and speak.")
            .await?;
    };
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(whitelisted_guilds)]
async fn queue(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.is_empty() {
        false => play(ctx, msg, args).await?,
        true => {
            let manager = songbird::get(ctx)
                .await
                .expect("Songbird Voice client placed in at initialisation.")
                .clone();

            // get our queue
            let data = ctx.data.read().await;
            let qu = &mut data.get::<VoiceQueue>().unwrap();
            let q = qu.write().await;

            let guild = msg.guild(&ctx.cache).unwrap();
            let guild_id = guild.id;

            if let Some(queue) = q.get(&guild_id) {
                let q = queue.current_queue();
                dbg!(&q);

                let mut to_send: String = "".to_string();
                if q.len() < 1 {
                    msg.channel_id.say(&ctx.http, "There isn't anything in the queue.").await?;
                    return Ok(());
                }
                for (i, track) in q.iter().enumerate() {
                    let info = track.metadata();
                    let none_value = "?";
                    let title = match &info.title {
                        Some(t) => t,
                        None => match &info.track {
                            Some(t) => t,
                            None => none_value,
                        },
                    };
                    let artist = match &info.artist {
                        Some(a) => a,
                        None => none_value,
                    };
                    let time = match &info.duration {
                        Some(d) => {
                            let mut time_str: String = "".to_string();
                            let mut time = d.as_secs();
                            if d.as_secs() > 3600 {
                                time_str.push_str(&format!("{}:", time / 3600));
                                time = time % 3600;
                            }
                            if d.as_secs() > 60 {
                                time_str.push_str(&format!("{}:", time / 60));
                                time = time % 60;
                                dbg!(time);
                            }
                            time_str.push_str(&format!("{:02}", time));
                            time_str
                        }
                        None => "?".to_string(),
                    };

                    let url = match &info.source_url {
                        Some(u) => {
                            let to_split = u.split("#?").collect::<Vec<_>>();
                            to_split[to_split.len() - 1].to_owned() + "\n"
                        }
                        None => "".to_string(),
                    };
                    let index = match i {
                        0 => "Now Playing".to_string(),
                        _ => i.to_string()
                    };


                    let to_push = format!("{}: {} - {} [{}]\n{}", index, title, artist, time, url);

                    to_send.push_str(&to_push);
                }
                msg.channel_id.say(&ctx.http, to_send).await?;
            }

            return Ok(());
        }
    };

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "You need to be in a voice channel.").await?;

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
#[checks(whitelisted_guilds)]
async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // get our queue
    let data = ctx.data.read().await;
    let qu = &mut data.get::<VoiceQueue>().unwrap();
    let q = qu.write().await;

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    if let Some(queue) = q.get(&guild_id) {
        let _ = queue.skip();

        msg.channel_id
            .say(
                &ctx.http,
                format!("1 song skipped in queue, {} left.", queue.len() - 1),
            )
            .await?;
    } else {
        msg.channel_id
            .say(&ctx.http, "You need to be in a voice channel for this.")
            .await?;
    }

    Ok(())
}
