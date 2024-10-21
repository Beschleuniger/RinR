#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::collections::HashMap;
use std::{env, thread};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use helper::{EventSignal, RinrOptions};
use tokio;
use dotenvy::dotenv;
use songbird::SerenityInit;
use serenity::model::prelude::Member;
use serenity::model::voice::VoiceState;
use serenity::{async_trait,
                 Client, client::*, 
                 prelude::{GatewayIntents, EventHandler},
                 model::{channel::*, prelude::Ready}};


mod helper;
use crate::helper::{fillStruct, checkDirs, readConfig, DailyEventSignalKey};

mod command;
use crate::command::{checkCommand, executeCommand,
                     User, COMMAND};

mod voice;
use crate::voice::joinVoice;

mod predict;
use crate::predict::UserPrediction;

mod timer;
// TODO: send audio file to discord channel command
// TODO: Image macro
// TODO: set bot channel info event system
// TODO: admin checked commands to wipe the sound folder


mod join;
use crate::join::resolveRoles;

mod poll;

mod fortnite;

mod event;
use crate::event::*;

//--------------------------------------------------------------------------------------------------------------------------
// Struct Declaration

struct Handler;

#[async_trait]
impl EventHandler for Handler {

    async fn ready(&self, ctx: Context, ready: Ready) {                // Successful connection to server check

        // Aquire Lock
        let mut u_data = ctx.data.write().await;           
        
        // Read config file
        let config: RinrOptions = readConfig().await;
        println!("Config: {:#?}", config);

        // Create channel for new thread
        let (send, recv): (Sender<EventSignal>, Receiver<EventSignal>) = mpsc::channel();

        let _event_handler = thread::spawn(move || {
            loops(config, recv);
        });



        u_data.insert::<DailyEventSignalKey>(send);


        // Gets saved Data
        let u_map: &mut HashMap<u64, UserPrediction> = u_data.get_mut::<User>().unwrap();
        let saved_map: HashMap<u64, UserPrediction> = fillStruct();



        if saved_map.is_empty() {
            println!("No saved data!");
        }

        for (key, value) in saved_map.iter() {                           // Merges hashmaps
            u_map.insert(*key, value.clone());
        } 

        println!("Loaded Predictions: {:#?}", u_map);

        println!("{}, Connected to Server!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        
        println!("{} said : {:?}", msg.author.name, msg.content);   // Debug: Shows message contents
        
        //if msg.author.id.0 == BOT_ID {return;}                    // Uncomment if you don't want the bot to execute commands it repeated using say

        let command: COMMAND = checkCommand(&msg).await;            // Checks if a message is a command

        if command == COMMAND::INVALID {return;}                    // Returns early if there is no command

        executeCommand(command, &msg, &ctx).await;
     
    }

    #[cfg(feature = "delete_annotation")]
    async fn message_delete(&self, ctx: Context, cid: ChannelId, _dmid: MessageId, gid: Option<GuildId>) {
        
        // Gets the last auditlog that has a member delete a message (72) and checks if the bot deleted it

        let action_type: Option<u8> = 72.into();
        let b_id: Option<u64> = None;
        let before: Option<u64> = None;
        let limit: Option<u8> = 1.into();

        let audit = ctx.http.get_audit_logs(gid.unwrap().0, action_type, b_id, before, limit).await;
        
        for a in audit.unwrap().entries {
            if a.target_id.unwrap() == BOT_ID {
                return;
            }
        }

        if let Err(why) = cid.say(&ctx.http, "Ich sehe das").await {
            println!("Send Message failed. Error: {:?}", why)
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        match joinVoice(ctx, old, &new).await {
            Ok(()) => println!("{} Successfully joined VC!", new.member.unwrap().user.name),
            Err(_) => (),
        };
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        
        match resolveRoles(ctx, &mut new_member.to_owned()).await {
            Ok(()) => println!("Successfully added Roles for user {}", new_member.user.name),
            Err(_) => println!("Unable to add role to user {}", new_member.user.name),
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
    | GatewayIntents::GUILDS
    | GatewayIntents::GUILD_VOICE_STATES
    | GatewayIntents::GUILD_PRESENCES
    | GatewayIntents::GUILD_MEMBERS
    | GatewayIntents::GUILD_MESSAGES
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
