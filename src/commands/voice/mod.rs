pub mod play;

use std::{collections::{HashMap, hash_map::RandomState}, sync::Arc};

use serenity::{model::id::GuildId, async_trait, http::Http, model::prelude::ChannelId, framework::standard::{Args, CommandOptions, Reason, macros::check}};
use serenity::model::prelude::*;
use serenity::prelude::*;

use songbird::{
    model::payload::{ClientConnect, ClientDisconnect, Speaking},
    Event, EventContext, EventHandler as VoiceEventHandler,
};
use crate::utils::queue::TrackQueue;

use tokio::sync::RwLock;

pub struct TrackEndNotifier {
    guild_id: GuildId,
    chan_id: ChannelId,
    http: Arc<Http>,
    //manager: Arc<Songbird>,
    queue: Arc<RwLock<HashMap<GuildId, TrackQueue, RandomState>>>,
}

#[check]
#[name = "whitelisted_guilds"]
async fn music_check(_: &Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> Result<(), Reason> {
    let allowed_guilds = [&228625269101953035, &290284538733658112, &781421814601089055, &418093857394262020, &381880193251409931];
    if !allowed_guilds.contains(&msg.guild_id.unwrap_or(GuildId(1)).as_u64()) {
        return Err(Reason::Log("Not in whitelisted guild".to_string()));
    }

    Ok(())
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_track_list) = ctx {
            let q = self.queue.write().await;

            let q_guild = q.get(&self.guild_id).unwrap();

            dbg!(q_guild.current()?.metadata());
            let m = q_guild.current()?.metadata();
            let message = match q_guild.len() {
                0 => "No songs left in queue.".to_string(),
                _ => match (&m.title, &m.artist) {
                    (Some(t), Some(a)) => format!("Now playing: {} by {}", t, a),
                    _ => "Now playing another song. (no metadata)".to_string()
                },
            };

            match self
                .chan_id
                .say(&self.http, &message)
                .await
            {
                Ok(e) => e,
                Err(e) => {
                    println!("Error sending message: {:?}", e);
                    return None;
                }
            };
        }
        None
    }
}

pub struct Receiver;

impl Receiver {
    pub fn new() -> Self {
        // You can manage state here, such as a buffer of audio packet bytes so
        // you can later store them in intervals.
        Self {}
    }
}

#[async_trait]
impl VoiceEventHandler for Receiver {
    #[allow(unused_variables)]
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        use EventContext as Ctx;

        match ctx {
            Ctx::SpeakingStateUpdate(Speaking {
                speaking,
                ssrc,
                user_id,
                ..
            }) => {
                // Discord voice calls use RTP, where every sender uses a randomly allocated
                // *Synchronisation Source* (SSRC) to allow receivers to tell which audio
                // stream a received packet belongs to. As this number is not derived from
                // the sender's user_id, only Discord Voice Gateway messages like this one
                // inform us about which random SSRC a user has been allocated. Future voice
                // packets will contain *only* the SSRC.
                //
                // You can implement logic here so that you can differentiate users'
                // SSRCs and map the SSRC to the User ID and maintain this state.
                // Using this map, you can map the `ssrc` in `voice_packet`
                // to the user ID and handle their audio packets separately.
                println!(
                    "Speaking state update: user {:?} has SSRC {:?}, using {:?}",
                    user_id, ssrc, speaking,
                );
            }
            Ctx::SpeakingUpdate { ssrc, speaking } => {
                // You can implement logic here which reacts to a user starting
                // or stopping speaking.
                println!(
                    "{} has {} speaking.",
                    ssrc,
                    if *speaking { "started" } else { "stopped" },
                );
            }
            Ctx::VoicePacket {
                audio,
                packet,
                payload_offset,
                payload_end_pad,
            } => {
                // An event which fires for every received audio packet,
                // containing the decoded data.
                if let Some(audio) = audio {
                    println!(
                        "Audio packet's first 5 samples: {:?}",
                        audio.get(..5.min(audio.len()))
                    );
                    println!(
                        "Audio packet sequence {:05} has {:04} bytes (decompressed from {}), SSRC {}",
                        packet.sequence.0,
                        audio.len() * std::mem::size_of::<i16>(),
                        packet.payload.len(),
                        packet.ssrc,
                    );
                } else {
                    println!("RTP packet, but no audio. Driver may not be configured to decode.");
                }
            }
            Ctx::RtcpPacket {
                packet,
                payload_offset,
                payload_end_pad,
            } => {
                // An event which fires for every received rtcp packet,
                // containing the call statistics and reporting information.
                //println!("RTCP packet received: {:?}", packet);
                ();
            }
            Ctx::ClientConnect(ClientConnect {
                audio_ssrc,
                video_ssrc,
                user_id,
                ..
            }) => {
                // You can implement your own logic here to handle a user who has joined the
                // voice channel e.g., allocate structures, map their SSRC to User ID.

                println!(
                    "Client connected: user {:?} has audio SSRC {:?}, video SSRC {:?}",
                    user_id, audio_ssrc, video_ssrc,
                );
            }
            Ctx::ClientDisconnect(ClientDisconnect { user_id, .. }) => {
                // You can implement your own logic here to handle a user who has left the
                // voice channel e.g., finalise processing of statistics etc.
                // You will typically need to map the User ID to their SSRC; observed when
                // speaking or connecting.

                println!("Client disconnected: user {:?}", user_id);
            }
            _ => {
                // We won't be registering this struct for any more event classes.
                unimplemented!()
            }
        }

        None
    }
}
