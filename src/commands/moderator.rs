extern crate chrono;
use chrono::prelude::*;

use chrono::Duration;
use chrono_humanize::HumanTime;

use two_timer::parse;

use serenity::framework::standard::macros::command;
use serenity::{
    framework::standard::{Args, CommandError, CommandResult},
    model::prelude::*,
    prelude::*,
    Error,
};
use crate::utils::user::get_members;

#[command]
#[required_permissions(MANAGE_MESSAGES)]
#[num_args(1)]
#[description("Prunes messages. (limit 99 at a time)")]
async fn prune(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let num = args.single::<u64>();
    match num {
        Err(_) => {
            msg.channel_id
                .say(ctx, "Value provided was not a number.")
                .await?;
        }
        Ok(n) => {
            let channel = &msg.channel(ctx).await.unwrap().guild().unwrap();

            let messages = &channel
                .messages(ctx, |r| r.before(&msg.id).limit(n))
                .await?;
            let messages_ids = messages.iter().map(|m| m.id).collect::<Vec<MessageId>>();

            match channel.delete_messages(ctx, messages_ids).await {
                Ok(()) => {
                    let returnmsg = msg
                        .channel_id
                        .say(ctx, format!("deleted `{}` messages", n))
                        .await?;
                    msg.delete(ctx).await?;
                    returnmsg.delete(ctx).await?;
                }
                Err(Error::Model(ModelError::InvalidPermissions(permissions))) => {
                    println!("{:?}", permissions);
                    return Err(CommandError::from("Missing Permissions: MANAGE_MESSAGES"));
                }
                Err(e) => {
                    println!("{:?}", e);
                    return Err(CommandError::from("Missing Permissions: MANAGE_MESSAGES"));
                }
            }
        }
    }
    Ok(())
}


#[command]
#[required_permissions(BAN_MEMBERS)]
#[description("Bans people. (limit one at a time)")]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let to_parse = args.single_quoted::<String>()?;
    let member = get_members(ctx, msg,to_parse).await;

    let reason = args.remains();

    match member {
        Ok(m) => {
            if let Some(r) = reason{
                m.ban_with_reason(ctx, 0,r).await?;
                msg.channel_id.say(ctx, format!("banned `{}` because `{}`", m.user.tag(), r)).await?;
            }
            else{
                m.ban(ctx, 0).await?;
                msg.channel_id.say(ctx, format!("banned `{}`, no reason given.", m.user.tag())).await?;
            }
        },
        Err(why) => {return Err(CommandError::from(why.to_string()))}
    }

    Ok(())
}

#[command]
#[required_permissions(KICK_MEMBERS)]
#[description("Kicks people. (limit one at a time)")]
async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let to_parse = args.single_quoted::<String>()?;
    let member = get_members(ctx, msg,to_parse).await;

    let reason = args.remains();

    match member {
        Ok(m) => {
            if let Some(r) = reason{
                m.kick_with_reason(ctx, r).await?;
                msg.channel_id.say(ctx, format!("kicked `{}` because `{}`", m.user.tag(), r)).await?;
            }
            else{
                m.kick(ctx).await?;
                msg.channel_id.say(ctx, format!("kicked `{}`, no reason given.", m.user.tag())).await?;
            }
        },
        Err(why) => {return Err(CommandError::from(why.to_string()))}
    }

    Ok(())
}

#[command]
#[required_permissions(KICK_MEMBERS)]
#[description("[WIP] Mute people. (limit one at a time)")]
async fn mute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    let to_parse = args.single_quoted::<String>()?;
    let member = get_members(ctx, msg,to_parse).await;

    let time_input = args.remains().unwrap_or("1hr").replace("until ", "");
    
    let time_raw = match parse(&time_input, None) {
        Ok((_d1, d2, _)) => Duration::seconds(d2.timestamp() - (Utc::now().timestamp_millis() / 1000)),
        Err(_) => {
            Duration::from_std(humantime::parse_duration(&time_input).unwrap_or(std::time::Duration::from_secs(11761200))).unwrap_or(Duration::hours(3267))},
    };

    let ht = Some(HumanTime::from(time_raw).to_string());

    if time_raw.num_minutes().is_negative() {
        return Err(CommandError::from(format!("h-A negative amount of time ({} | {}) was given.", ht.unwrap(), time_raw)));
    } else if time_raw.num_minutes() == 196020 {
        return Err(CommandError::from("h-Could not parse anything."));
    }

    match member {
        Ok(m) => {
            if let Some(ht) = ht{
                msg.channel_id.say(ctx, format!("muted `{}`. they will be unmuted `{}`", m.user.tag(), ht)).await?;
            }
            else{
                msg.channel_id.say(ctx, format!("muted `{}`, no reason given.", m.user.tag())).await?;
            }
        },
        Err(why) => {return Err(CommandError::from(why.to_string()))}
    }

    Ok(())
}

