use serenity::{model::id::GuildId, client::bridge::gateway::ShardManager};

use serenity::prelude::*;

use std::{collections::HashMap, sync::Arc};

use sqlx::PgPool;

use tokio::sync::Mutex;

use chrono::{DateTime, Utc};

use crate::utils::queue::TrackQueue;

use tokio::sync::RwLock;

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct Uptime;
impl TypeMapKey for Uptime {
    type Value = HashMap<String, DateTime<Utc>>;
}

pub struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}

pub struct VoiceQueue;

impl TypeMapKey for VoiceQueue {
    type Value = Arc<RwLock<HashMap<GuildId, TrackQueue>>>;
}
