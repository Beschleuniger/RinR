#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::{env};

use serenity::prelude::TypeMapKey;
use serenity::{async_trait, Client, client::*, prelude::{GatewayIntents, EventHandler}, model::{channel::*, prelude::{Ready}}};
use tokio;
use strum::{IntoEnumIterator, EnumCount};
use strum_macros::{EnumIter, Display};
use regex::Regex;
use rustube;

//--------------------------------------------------------------------------------------------------------------------------
// Const Declaration

const TEST: &str = "$test ";
const SET: &str = "$setvideo ";
const LIST: &str = "$list ";
const DISCONNECT: &str = "$disconnect ";
const STFU: &str = "STFU ";
const KYS: &str = "kys ";
const TIMER: &str = "$timer ";
const WIN: &str = "$win ";
const BAN: &str = "$ban ";
const ULIST: &str = "$userlist ";

static CONSTS: &'static [&str] = &[TEST, SET, LIST, DISCONNECT, STFU,
                                     KYS, TIMER, WIN, BAN, ULIST];


const TEST_RESPONSE: &str = "Pissing all by yourself handsome?";
const SET_RESPONSE: &str = "New video set!";

//--------------------------------------------------------------------------------------------------------------------------
// Enum Declaration 


#[derive(Debug, EnumIter, Display, EnumCount, PartialEq)]
enum COMMAND {
    E_TEST,
    E_SET,
    E_LIST,
    E_DISCONNECT,
    E_STFU,
    E_KYS,
    E_TIMER,
    E_WIN,
    E_BAN,
    E_ULIST,
    INVALID,
}

//--------------------------------------------------------------------------------------------------------------------------
// Struct Declaration

struct Handler;

#[async_trait]
impl EventHandler for Handler {

    async fn message(&self, ctx: Context, msg: Message) {
        
        println!("Message Contains: {:?}", msg.content);    // Debug: Shows message contents
        
        let command: COMMAND = checkCommand(&msg);          // Checks if a message is a command

        if command == COMMAND::INVALID {return;}            // Returns early if there is no command

        executeCommand(command, &msg, &ctx);            // Executes Commands 

    }

    async fn ready(&self, _: Context, ready: Ready){        // Successful connection to server check
        println!("{}, Connected to Server!", ready.user.name);
    }
}

#[derive(Debug)]
struct User {}

impl TypeMapKey for User {
    type Value = HashMap<u64, String>;
}

async fn insert_user(ctx: &Context, _msg: &Message) {  
    let mut u_data = ctx.data.write().await;            // Waits for Lock Queue on write command and then proceeds with execution 
    let u_map = u_data.get_mut::<User>().unwrap();    // Gets mutable reference to the data and stores it in counter
    
    // Should checks pass
    // Check for additional info
    // Download video
    // Trim Video
    // Save Video and get filepath to it (user ID as title)
    // Add the User to the Map or replace their filepath (map value)



    u_map.insert(12313123123, "Sex".to_string());                             // Inserts Element into Map
    println!("{:?}", u_map);
}

//--------------------------------------------------------------------------------------------------------------------------

fn checkCommand(msg: &Message) -> COMMAND {

    // Makes sure that the enum matches the consts
    if CONSTS.len() > (COMMAND::COUNT - 1) {
        println!("Refractor Enum or Consts!");
        return COMMAND::INVALID;
    }

    let mut command: COMMAND = COMMAND::INVALID;
    let cmd_iter: COMMANDIter = COMMAND::iter(); // Creates iterator for enum, WHICH DOESN'T IMPLEMENT DEBUG BY THE WAY FOR SOME UNGODLY REASON

    for (pos, c)  in CONSTS.iter().enumerate() {
        
        if msg.content.clone().starts_with(c) {
                        
            command = cmd_iter.get(pos).unwrap_or(COMMAND::INVALID);

            break;
        }
    }

    command
} 

fn executeCommand(cmd: COMMAND, msg: &Message, ctx: &Context) {

    match cmd {
        COMMAND::E_TEST => println!("Test!"),
        COMMAND::E_SET => userMapCheckAndUpdate(&msg, &ctx),
        COMMAND::INVALID => (),                                     // Should never happen but better be safe than sorry
        _ => println!("Not Implemented Yet"),
    }
}

fn userMapCheckAndUpdate(msg: &Message, _ctx: &Context) {
    
    let reg: Regex = Regex::new(r"https://(?:www)?\.?youtu\.?be\.?(?:com)?/?(?:watch\?v=)?(.{11})").unwrap();
    let mut yt: &str = msg.content.as_str().clone();

    match reg.captures(yt) {
        Some(capture) => yt = capture.get(1).unwrap().as_str(),
        None => println!("Nothing Captured!"),
    }



}


#[tokio::main]
async fn main() {

    // Sets DC Token as ENV
    let key = "TOKEN";
    env::set_var(key, "OTA5NTY3ODM3OTY0NzQ2ODYz.YZGLDw.NcBgor98gTrtdwJsdfUZRtq79gs");

    // Lets bot know which event it should listen to
    let intents = GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT;

    // Assigns Token
    let token: String = env::var(key)
        .expect("No Token Given!");

    // Creates Client 
    let mut client: Client = Client::builder(token, intents)
        .event_handler(Handler)
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
