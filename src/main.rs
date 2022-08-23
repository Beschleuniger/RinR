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
use rustube::{VideoFetcher, Id};

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

const YT: &str = "https://youtu.be/";

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
        
        println!("{} said : {:?}", msg.author, msg.content);        // Debug: Shows message contents
        
        let command: COMMAND = checkCommand(&msg).await;            // Checks if a message is a command

        if command == COMMAND::INVALID {return;}                    // Returns early if there is no command

        executeCommand(command, &msg, &ctx).await;              // Executes Commands 

    }

    async fn ready(&self, _: Context, ready: Ready){                // Successful connection to server check
        println!("{}, Connected to Server!", ready.user.name);
    }
}

#[derive(Debug)]
struct User {}

impl TypeMapKey for User {
    type Value = HashMap<u64, String>;
}


#[derive(Debug)]
struct VidInfo {
    name: String,       // Video name
    v_length: u64,      // Video length          
    start: u64,         // User start point     Default: 0
    u_length: u64,      // Clip length          Default: 5
}

//--------------------------------------------------------------------------------------------------------------------------

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

async fn checkCommand(msg: &Message) -> COMMAND {

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

async fn executeCommand(cmd: COMMAND, msg: &Message, ctx: &Context) {

    match cmd {
        COMMAND::E_TEST => println!("Test!"),
        COMMAND::E_SET => userMapCheckAndUpdate(&msg, &ctx).await,
        COMMAND::INVALID => (),                                     // Should never happen but better be safe than sorry
        _ => println!("Not Implemented Yet"),
    }
}

async fn userMapCheckAndUpdate(msg: &Message, ctx: &Context) {
    
    let reg: Regex = Regex::new(r"https://(?:www)?\.?youtu\.?be\.?(?:com)?/?(?:watch\?v=)?(.{11})").unwrap();   // Regex to match YouTube links (long and short urls work / YouTube Shorts don't)
    let mut yt: String = msg.content.clone();
    let mut vid: VidInfo = VidInfo {name: "".to_string(), v_length: 0, start: 0, u_length: 0,};

    match reg.captures(&yt) {   // Uses Regex to capture the 11 URL characters that are important
        Some(capture) => yt = capture.get(1).unwrap().as_str().to_string().to_owned(),  
        None => {errHandle(msg, ctx, 0).await; return;},
    }

    let id = match Id::from_str(&yt) {  // Does it again, but this time its from the api
        Ok(T) => T,
        Err(_E) => {errHandle(msg, ctx, 0).await; return;},
    };

    let descrambler = match VideoFetcher::from_id(id.into_owned()) // Fetches Videoinfo, should it exists
        .unwrap()
        .fetch()
        .await {
            Ok(T) => T,
            Err(_E) => {errHandle(msg, ctx, 0).await; return;},
        };

    let info = descrambler.video_info();    // Saves video info in variable
    vid.name = info.player_response.video_details.title.clone();
    vid.v_length = info.player_response.video_details.length_seconds.clone();

    
    vid.start = matchStart(&msg.content.as_str(), &vid.v_length).await;
    vid.u_length = matchLength(&msg.content.as_str(), &vid.v_length, &vid.start).await;


    println!("{:#?}", &vid);

}

async fn errHandle(msg: &Message, ctx: &Context, case: u8) {

    match case {

        0 => if let Err(why) = msg.channel_id.say(&ctx.http, "No valid Youtube Link given!").await {
            println!("Send Message failed. Error: {:?}", why)
            },
        1 => (),
        2 => (),
        _ => println!("Invalid Case! / Not Implemented!"),
    }

}

async fn matchStart(msg: &str, v_length: &u64) -> u64 {

    let start: Regex = Regex::new(r"start=([0-9]+)").unwrap();                                                  // Optional Regex to match the start point

    let vid_start: u64 = match start.captures(&msg) {   
        Some(capture) => {
            if capture.get(1).unwrap().as_str().to_string().to_owned().clone().parse::<u64>().unwrap() >=  *v_length {
                v_length - 2   // Checks if custom start is longer than video len, if yes assigns it to max vid length - 2
            } else {
                capture.get(1).unwrap().as_str().to_string().to_owned().parse::<u64>().unwrap()
            }
        },  
        None => 0,  // No match == start at 0
    };

    vid_start
}

async fn matchLength(msg: &str, v_length: &u64, v_start: &u64) -> u64 {
    
    let length: Regex =  Regex::new(r"length=([0-9]{1,2})").unwrap();                                           // Optional Regex to match the clip length
    
    let u_length: u64 = match length.captures(&msg) {   
        Some(capture) => {
            let mut len: u64 = capture.get(1).unwrap().as_str().to_string().to_owned().clone().parse::<u64>().unwrap();

            if  len > 10 { len = 10; }           // Makes sure length is in the right size bracket 
            else if len < 1 {len = 1; }

            if  len + v_start >= *v_length {
                len = v_length - v_start;  // Checks if the custom length would go over the video length and, if appropriate, sets it to the remaining time in the video
            } 

            len
        },  
        None => {
            let mut len: u64 = 5;   // Default length of 5

            if v_start + len > *v_length {
                len = v_length - v_start;
            }

            len
        }
    };

    u_length
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
