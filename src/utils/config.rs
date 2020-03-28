use reqwest;
use serenity::prelude::Context;
use std::sync::Arc;
use crate::keys::Reqwest;

pub fn get_reqwest_client(ctx: &Context) -> Arc<reqwest::blocking::Client> {
    let data = ctx.data.read();
    data.get::<Reqwest>().unwrap().clone()
}