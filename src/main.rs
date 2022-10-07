#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::{env};

use serenity::model::prelude::{ChannelId, MessageId, GuildId};
use serenity::model::voice::VoiceState;
use serenity::{async_trait, Client, client::*, prelude::{GatewayIntents, EventHandler}, model::{channel::*, prelude::Ready}};
use songbird::SerenityInit;
use tokio;
use dotenv::dotenv;

mod helper;
use crate::helper::*;

mod command;
use crate::command::*;

mod voice;
use crate::voice::*;

// TODO: send audio file to discord channel command
// TODO: Update Readme and get icon/logo
// TODO: Image macro
// TODO: On mention "wos wüast du hurensohn"


//--------------------------------------------------------------------------------------------------------------------------
// Struct Declaration

struct Handler;

#[async_trait]
impl EventHandler for Handler {

    async fn message(&self, ctx: Context, msg: Message) {
        
        println!("{} said : {:?}", msg.author.name, msg.content);   // Debug: Shows message contents
        
        let command: COMMAND = checkCommand(&msg).await;            // Checks if a message is a command

        if command == COMMAND::INVALID {return;}                    // Returns early if there is no command

        executeCommand(command, &msg, &ctx).await;              // Executes Commands 

    }

    async fn ready(&self, ctx: Context, ready: Ready){                // Successful connection to server check

        let mut u_data = ctx.data.write().await;           
        let u_map: &mut HashMap<u64, String> = u_data.get_mut::<User>().unwrap();
        let saved_map: HashMap<u64, String> = fillStruct();           // Gets saved Data

        if saved_map.is_empty() {
            println!("No saved data!");
        }

        for (key, value) in saved_map.iter() {                           // Merges hashmaps
            u_map.insert(*key, value.to_string());
        } 

        println!("Current Users: {:#?}", u_map);

        println!("{}, Connected to Server!", ready.user.name);
    }

    async fn message_delete(&self, ctx: Context, cid: ChannelId, _mid: MessageId, _guildid: Option<GuildId>) {

        if let Err(why) = cid.say(&ctx.http, "Ich sehe das").await {
            println!("Send Message failed. Error: {:?}", why)
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        
        match joinVoice(ctx, old, new).await {
            Ok(()) => println!("Successfully joined VC!"),
            Err(_) => (),
        };
    }

}


#[tokio::main]
async fn main() {

    // Sets DC Token as ENV
    dotenv().expect("Please provide a .env file with your Bot Token!");

    // Checks if dirs exist and creates them if not
    checkDirs().await;

    // Lets bot know which event it should listen to
    let intents = GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT
    | GatewayIntents::GUILD_VOICE_STATES
    | GatewayIntents::GUILD_PRESENCES
    | GatewayIntents::GUILD_MEMBERS
    | GatewayIntents::non_privileged();

    // Assigns Token
    let token: String = env::var("TOKEN")
        .expect("No Token Given!");

    // Creates Client 
    let mut client: Client = Client::builder(token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Couldn't Create Client!");
        {
            let mut u_data = client.data.write().await;
            u_data.insert::<User>(HashMap::default());
        }
 
    // Connects to Server
    if let Err(r) = client.start().await {
         println!("Client Error: {:?}", r);
    }
}
