#![warn(non_snake_case)]
#![allow(non_camel_case_types)]

use std::env;

use serenity::{async_trait, Client, client::*, prelude::{GatewayIntents, EventHandler}, model::{channel::*, prelude::Ready}};
use tokio;


struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, _msg: Message) {
        if false {
            panic!();
        }
    }

    async fn ready(&self, _: Context, _ready: Ready){
        println!("Connected to Server!");
    }
}

//test

#[tokio::main]
async fn main() {

    let key = "TOKEN";
    env::set_var(key, "OTA5NTY3ODM3OTY0NzQ2ODYz.YZGLDw.NcBgor98gTrtdwJsdfUZRtq79gs");

    // TODO: Add Token file
    let token: String = env::var(key)
        .expect("No Token Given!");

    let mut client: Client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Couldn't Create Client!");


    if let Err(r) = client.start().await {
         println!("Client Error: {:?}", r);
    }
}
