use serenity::framework::standard::macros::command;
use serenity::{
    framework::standard::{Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
#[required_permissions(MANAGE_MESSAGES)]
#[num_args(1)]
#[description("Prunes messages. (limit 99 at a time)")]
async fn prune(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let num = args.single::<u64>();
    match num {
        Err(_) => {msg.channel_id.say(ctx, "The value provided was not a valid number").await?;},
        Ok(n) => {
            let channel = &msg.channel(ctx).await.unwrap().guild().unwrap();

            let messages = &channel.messages(ctx, |r| r.before(&msg.id).limit(n)).await?;
            let messages_ids = messages.iter().map(|m| m.id).collect::<Vec<MessageId>>();

            channel.delete_messages(ctx, messages_ids).await?;
            let returnmsg = msg.channel_id.say(ctx, format!("deleted `{}` messages", n)).await?;
            msg.delete(ctx).await?;
            returnmsg.delete(ctx).await?;
        }
    }
    Ok(())
}
