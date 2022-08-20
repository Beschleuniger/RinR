#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::env;

use serenity::{async_trait, Client, client::*, prelude::{GatewayIntents, EventHandler}, model::{channel::*, prelude::Ready}};
use tokio;

//--------------------------------------------------------------------------------------------------------------------------
// Const Declaration

const TEST: &str = "$test";
const SET: &str = "$setvideo";
const LIST: &str = "$list";
const DISCONNECT: &str = "$disconnect";
const STFU: &str = "STFU";
const KYS: &str = "kys";
const TIMER: &str = "$timer";
const WIN: &str = "$win";
const BAN: &str = "$ban";
const ULIST: &str = "$userlist";

static CONSTS: &'static [&str] = &[TEST, SET, LIST, DISCONNECT, STFU,
                                     KYS, TIMER, WIN, BAN, ULIST];


const TEST_RESPONSE: &str = "Pissing all by yourself handsome?";
const SET_RESPONSE: &str = "New video set!";

//--------------------------------------------------------------------------------------------------------------------------
// Enum Declaration 

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

struct Handler;

#[async_trait]
impl EventHandler for Handler {

    async fn message(&self, ctx: Context, msg: Message) {
        
        println!("{:?}", msg.content);
        
        let command: COMMAND = checkCommand(&msg);


    }

    async fn ready(&self, _: Context, ready: Ready){
        println!("{}, Connected to Server!", ready.user.name);
    }
}


fn checkCommand(msg: &Message) -> COMMAND {

    let command: COMMAND = COMMAND::INVALID;




    command
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

    // Connects to Server
    if let Err(r) = client.start().await {
         println!("Client Error: {:?}", r);
    }
}
