#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::{env};

use serenity::prelude::TypeMapKey;
use serenity::{async_trait, Client, client::*, prelude::{GatewayIntents, EventHandler}, model::{channel::*, prelude::{Ready}}};
use tokio;
use tokio::sync::Mutex;
use strum::{IntoEnumIterator, EnumCount};
use strum_macros::{EnumIter, Display};


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


#[derive(Debug, EnumIter, Display, EnumCount)]
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
        
        println!("Message Contains: {:?}", msg.content);
        
        let command: COMMAND = checkCommand(&msg);

        test(&ctx).await;

        executeCommand(command, &msg, &ctx);

    }

    async fn ready(&self, _: Context, ready: Ready){
        println!("{}, Connected to Server!", ready.user.name);
    }
}

#[derive(Debug)]
struct User {}

impl TypeMapKey for User {
    type Value = HashMap<u64, String>;
}

async fn test(ctx: &Context) {
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<User>().unwrap();
    counter.insert(102984572, "Sex".to_string());
    println!("{:?}", counter);
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
        COMMAND::E_SET => println!("Putis Function here"),
        COMMAND::INVALID => (),
        _ => println!("Not Implemented Yet"),
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
            let mut data = client.data.write().await;
            data.insert::<User>(HashMap::default());
        }
 
    // Connects to Server
    if let Err(r) = client.start().await {
         println!("Client Error: {:?}", r);
    }
}
