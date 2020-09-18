use serenity::{
    model::prelude::*,
    prelude::*,
};
use regex::Regex;
use futures::{
    stream,
    StreamExt,
};


pub fn get_id(value: &str) -> Option<u64> {
    // check if it's already an ID
    if let Ok(id) = value.parse::<u64>() {
        return Some(id);
    }

    // from https://docs.rs/serenity/0.4.5/src/serenity/utils/mod.rs.html#158-172
    if value.starts_with("<@!") {
        let len = value.len() - 1;
        value[3..len].parse::<u64>().ok()
    } else if value.starts_with("<@") {
        let len = value.len() - 1;
        value[2..len].parse::<u64>().ok()
    } else {
        None
    }
}

//stolen from https://gitlab.com/nitsuga5124/robo-arc/-/blob/master/src/commands/moderation.rs
pub async fn get_members(ctx: &Context, msg: &Message, member: String) -> Result<Member, String> {
    let mut members: Vec<&Member> = Vec::new();
    if let Ok(id) = member.parse::<u64>(){
        // gets a member from user id
        let member = &msg.guild_id.unwrap().member(ctx, id).await;
        match member {
            Ok(m) => Ok(m.to_owned()),
            Err(why) => Err(why.to_string()),
        }
    } else if member.starts_with("<@") && member.ends_with(">"){
        let re = Regex::new("[<@!>]").unwrap();
        let member_id = re.replace_all(&member, "").into_owned();
        let member = &msg.guild_id.unwrap().member(ctx, UserId(member_id.parse::<u64>().unwrap())).await;
        match member {
            Ok(m) => Ok(m.to_owned()),
            Err(why) => Err(why.to_string()),
        }

    } else {
        let guild = &msg.guild(ctx).await.unwrap();
        let member = member.split('#').next().unwrap();

        for m in guild.members.values() {
            if m.display_name() == std::borrow::Cow::Borrowed(member) ||
                m.user.name == member
            {
                members.push(m);
            }
        }

        if members.is_empty() {
            let similar_members = &guild.members_containing(&member, false, false).await;

            let mut members_string =  stream::iter(similar_members.iter())
                .map(|m| async move {
                    let member = &m.0.user;
                    format!("`{}`|", member.name)
                })
                .fold(String::new(), |mut acc, c| async move {
                    acc.push_str(&c.await);
                    acc
                }).await;

            let message = {
                if members_string == "" {
                    format!("No member named '{}' was found.", member.replace("@", ""))
                } else {
                    members_string.pop();
                    format!("No member named '{}' was found.\nDid you mean: {}", member.replace("@", ""), members_string.replace("@", ""))
                }
            };
            Err(message)
        } else if members.len() == 1 {
            Ok(members[0].to_owned())
        } else {
            let mut members_string =  stream::iter(members.iter())
                .map(|m| async move {
                    let member = &m.user;
                    format!("`{}#{}`|", member.name, member.discriminator)
                })
                .fold(String::new(), |mut acc, c| async move {
                    acc.push_str(&c.await);
                    acc
                }).await;

            members_string.pop();

            let message = format!("Multiple members with the same name were found. Try again with the users' name, id, or a more accurate string.\n`{}`", &members_string);
            Err(message)
        }
    }
}