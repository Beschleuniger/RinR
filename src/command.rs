use std::collections::HashMap;
use std::{path::Path, process::Command};
use std::io::Write;
use std::{fs, fs::File};
use std::fmt::Error;

use strum::{IntoEnumIterator, EnumCount};
use strum_macros::{EnumIter, Display};
use regex::Regex;
use rustube::{VideoFetcher, Id};
use hrtime;

use serenity::{client::*, model::{channel::*}};
use serenity::prelude::TypeMapKey;

use crate::helper::*;


//--------------------------------------------------------------------------------------------------------------------------
// Const Declaration

static TEST: &str = "$test ";
static SET: &str = "$setvideo ";
static LIST: &str = "$list ";
static DISCONNECT: &str = "$disconnect ";
static STFU: &str = "STFU ";
static KYS: &str = "kys ";
static TIMER: &str = "$timer ";
static WIN: &str = "$win ";
static BAN: &str = "$ban ";
static ULIST: &str = "$userlist ";

static TEST_RESPONSE: &str = "Pissing all by yourself handsome?";
static SET_RESPONSE: &str = "New video set!\nFor User: ";

static YT: &str = "https://youtu.be/";

static CONSTS: &'static [&str] = &[TEST, SET, LIST, DISCONNECT, STFU,
                                     KYS, TIMER, WIN, BAN, ULIST];


//--------------------------------------------------------------------------------------------------------------------------
// Enum Declaration 

#[derive(Debug, EnumIter, Display, EnumCount, PartialEq)]
pub enum COMMAND {
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

#[derive(Debug)]
pub struct VidInfo {
    name: String,       // Video name
    v_length: u64,      // Video length          
    start: u64,         // User start point     Default: 0
    u_length: u64,      // Clip length          Default: 5
    u_id: String,       // User Id
}


#[derive(Debug)]
pub struct User {}

impl TypeMapKey for User {
    type Value = HashMap<u64, String>;
}


//--------------------------------------------------------------------------------------------------------------------------
// Checks message against list of commands
pub async fn checkCommand(msg: &Message) -> COMMAND {
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


//--------------------------------------------------------------------------------------------------------------------------
// Matches commands with the functions they should execute
pub async fn executeCommand(cmd: COMMAND, msg: &Message, ctx: &Context) {

    match cmd {
        COMMAND::E_TEST => println!("Test!"),
        COMMAND::E_SET => userMapCheckAndUpdate(&msg, &ctx).await,
        COMMAND::INVALID => (),                                     // Should never happen 
        _ => println!("Not Implemented Yet"),
    }
}


//--------------------------------------------------------------------------------------------------------------------------
// Adds user to user struct
async fn insert_user(ctx: &Context, msg: &Message) {  
    let mut u_data = ctx.data.write().await;           // Waits for Lock Queue on write command and then proceeds with execution 
    let u_map: &mut HashMap<u64, String> = u_data.get_mut::<User>().unwrap();    // Gets mutable reference to the data and stores it in counter
    
    let key: u64 = msg.author.id.0;
    let path: String = buildVidPath(key.clone().to_string());

    let filepath: String = buildTxtPath();

    u_map.insert(key, path);                             // Inserts Element into Map
    println!("Full Map: {:#?}", u_map);

    match fs::remove_file(&filepath) {
        Ok(()) => (),
        Err(_) => println!("Couldn't delete Struct File!"),
    }

    let mut file: File  = match File::create(&filepath) {
        Ok(O) => O,
        Err(_) => return,
    };

    for user in u_map {
        
        let mut test: String = user.0.to_string();
        test.push_str("=");
        test.push_str(user.1.as_str());
        test.push_str("\n");

        file.write_all(test.as_bytes()).expect("Couldn't write to File!");

    }
}


//--------------------------------------------------------------------------------------------------------------------------
// Handles most of the logic for the YouTube video detection 
async fn userMapCheckAndUpdate(msg: &Message, ctx: &Context) {

    if let Err(why) = msg.channel_id.say(&ctx.http, "Aight").await {
        println!("Send Message failed. Error: {:?}", why)
    }

    // Sets filepath
    let u_name: String = removeUserAt(msg.author.id.0.to_string());
    let path: &Path = Path::new(u_name.as_str());

    println!("Path to File: {:?}", path);


    // Sets and matches YouTube Regex
    let reg: Regex = Regex::new(r"https://(?:www)?\.?youtu\.?be\.?(?:com)?/?(?:watch\?v=)?(.{11})").unwrap();   // Regex to match YouTube links (long and short urls work / YouTube Shorts don't)
    let mut yt: String = msg.content.clone();
    let mut vid: VidInfo = VidInfo {name: "".to_string(), v_length: 0, start: 0, u_length: 0, u_id: "".to_string(),};

    match reg.captures(&yt) {   // Uses Regex to capture the 11 URL characters that are important
        Some(capture) => yt = capture.get(1).unwrap().as_str().to_string().to_owned(),  
        None => {errHandle(msg, ctx, 0).await; return;},
    }

    let id = match Id::from_str(&yt) {  // Does it again, but this time its from the api
        Ok(T) => T,
        Err(_) => {errHandle(msg, ctx, 0).await; return;},
    };


    // Starts a descrambler for the Video Data
    let descrambler = match VideoFetcher::from_id(id.into_owned()) // Fetches Videoinfo, should it exists
        .unwrap()
        .fetch()
        .await {
            Ok(T) => T,
            Err(_) => {errHandle(msg, ctx, 0).await; return;},
        };

    let info = descrambler.video_info();    // Saves video info in variable


    // Saves some of the video info in a better format
    vid.name = info.player_response.video_details.title.clone();
    vid.v_length = info.player_response.video_details.length_seconds.clone();
    vid.start = matchStart(&msg.content.as_str(), &vid.v_length).await;
    vid.u_length = matchLength(&msg.content.as_str(), &vid.v_length, &vid.start).await;
    vid.u_id = msg.author.id.0.to_string().clone();

    println!("{:#?}", &vid);


    // Tries to download video to location
    match descrambler
        .descramble()
        .unwrap()
        .best_audio()
        .unwrap()
        .download_to(path)
        .await {
            Ok(()) => println!("Successful Download!"),
            Err(_) => {errHandle(msg, ctx, 1).await; return;}
        };

    // Tries to trim the video
    match editVideo(&vid).await {
        Ok(()) => println!("Successful Edit!"),
        Err(_) => {errHandle(msg, ctx, 2).await; return;},
    };

    // Adds to user struct
    insert_user(ctx, msg).await;

    let mut response: String = SET_RESPONSE.to_string();
    response.push_str(msg.author.name.as_str());

    if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
        println!("Send Message failed. Error: {:?}", why)
    }
}


//--------------------------------------------------------------------------------------------------------------------------
// Trims the video file
async fn editVideo(vid: &VidInfo) -> Result<(), Error> {

    let path: String = removeUserAt(vid.u_id.clone());
    let path_edit: String = path.clone().replace(".mp3", "_edit.mp3");

    let start: String = hrtime::from_sec_padded(vid.start);
    let end: String = hrtime::from_sec_padded(vid.start + vid.u_length);
    
    match fs::remove_file(&path_edit) {
        Ok(()) => (),
        Err(_) => println!("No File to delete! / No Permission to delete File!"),
    }

    match Command::new("ffmpeg").args(["-i", path.as_str(), "-ss", start.as_str(), "-to", end.as_str(), path_edit.as_str()]).output() {
        Ok(O) => {
            match fs::remove_file(path) {
                Ok(()) => (),
                Err(_) => println!("Couldn't delete Source File!"),
            }
            println!("Stdout: {:?}", O.stdout);
        }
        Err(_) => return Err(Error), 
    };
    

    Ok(())
}



//--------------------------------------------------------------------------------------------------------------------------
// Error Handler 
async fn errHandle(msg: &Message, ctx: &Context, case: u8) {

     let err: &str = match case {

        0 => "No valid Youtube Link given!",
        1 => "Unable to downlaod Video!",
        2 => "Unable to edit video, old video has been overwritten!",
        _ => "Invalid Case! / Not Implemented!",
    };

    if let Err(why) = msg.channel_id.say(&ctx.http, err).await {
        println!("Send Message failed. Error: {:?}", why)
    }

}


//--------------------------------------------------------------------------------------------------------------------------
// Matches custom start time for a youtube video
async fn matchStart(msg: &str, v_length: &u64) -> u64 {

    let start: Regex = Regex::new(r"start=([0-9]+)").unwrap();              // Optional Regex to match the start point

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


//--------------------------------------------------------------------------------------------------------------------------
// Matches custom length for a youtube video
async fn matchLength(msg: &str, v_length: &u64, v_start: &u64) -> u64 {
    
    let length: Regex =  Regex::new(r"length=([0-9]{1,2})").unwrap();       // Optional Regex to match the clip length
    
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
