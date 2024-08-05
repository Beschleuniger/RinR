use crate::helper::*;

use serenity::{client::Context, model::prelude::Message};

use fortnite_api::{get_shop_combined_v2, response_types::shop::ShopV2};

use reqwest::Client;


pub async fn runFortnite(msg: &Message, ctx: &Context){

    let http_client: Client = reqwest::Client::new();

    let result: reqwest::Result<ShopV2> = get_shop_combined_v2(&http_client, None).await;

    match result {
        Ok(Shop) => say(msg, ctx, Shop.date.to_string()).await,
        Err(e) => say(msg, ctx, e.to_string()).await,
    };

}