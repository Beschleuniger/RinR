use std::collections::HashMap;
use std::{path::Path, process::Command};
use std::fmt::Error;

#[cfg(feature = "old_downloader")]
use std::fs;

use tokio::task;
use strum::{IntoEnumIterator, EnumCount};
use strum_macros::{EnumIter, Display};
use regex::Regex;

#[cfg(feature = "old_downloader")]
use rustube::{VideoFetcher, Id};

#[cfg(feature = "old_downloader")]
use hrtime;

use serenity::{client::*, model::channel::*};
use serenity::prelude::TypeMapKey;
use youtube_dl::YoutubeDl;

use crate::helper::*;
use crate::predict::*;
use crate::timer::*;
use crate::poll::*;
use crate::fortnite::*;
use crate::event::*;
use crate::santa::santaHandler;


//--------------------------------------------------------------------------------------------------------------------------
// Const Declaration

const TEST: &str = "$test";
const SET: &str = "$setvideo ";
const LIST: &str = "$list ";
const DISCONNECT: &str = "$disconnect ";
const STFU: &str = "STFU";
const KYS: &str = "kys";
const TIMER: &str = "$timer ";
const WIN: &str = "$win";
const BAN: &str = "$ban ";
const ULIST: &str = "$userlist ";
const SAY: &str = "$say ";
pub const PREDICTION: &str = "$predict ";
const POLL: &str = "$poll ";
const FORTNITE: &str = "$fn ";
const EVENT: &str = "$event ";
const SANTA: &str = "$santa";

//const TEST_RESPONSE: &str = "Pissing all by yourself handsome?";
const SET_RESPONSE: &str = "New video set!\nFor User: ";

//const YT: &str = "https://youtu.be/";

const CONSTS: &'static [&str] = &[TEST, SET, LIST, DISCONNECT, STFU,
                                  KYS, TIMER, WIN, BAN, ULIST, SAY,
                                  PREDICTION, POLL, FORTNITE, EVENT, SANTA];


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
    E_SAY,
    E_PREDICTION,
    E_POLL,
    E_FORTNITE,
    E_EVENT,
    E_SANTA,
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
    type Value = HashMap<u64, UserPrediction>;
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
        COMMAND::E_TIMER => timer(&msg, &ctx).await,
        COMMAND::E_WIN => winOrLose(&msg, &ctx).await,
        COMMAND::E_SAY => repeatMessage(&msg, &ctx).await,
        COMMAND::E_PREDICTION => addPrediction(&msg, &ctx).await,
        COMMAND::E_POLL => runPoll(&msg, &ctx).await,
        COMMAND::E_FORTNITE => fortniteWrapper(&msg, &ctx).await,
        COMMAND::E_EVENT => eventHandler(&msg, &ctx).await,
        COMMAND::E_SANTA => santaHandler(&msg, &ctx).await,
        COMMAND::INVALID => (),                                     // Should never happen 
        _ => println!("Not Implemented Yet"),
    }
}

//--------------------------------------------------------------------------------------------------------------------------
// gives a 50/50 chance for a win or a loss
async fn winOrLose(msg: &Message, ctx: &Context) {
    
    let rng: bool = rand::random();
    let out: &str = if rng {"W"} else {"L"};

    say(msg, ctx, out.to_string()).await;

}

//--------------------------------------------------------------------------------------------------------------------------
// Repeats Message sent by user and deletes their message
async fn repeatMessage(msg: &Message, ctx: &Context) {

    let out: String = msg.content.clone().replace(SAY, "");

    say(msg, ctx, out).await;

    delete(msg, ctx).await;

}


//--------------------------------------------------------------------------------------------------------------------------
// Handles download of a video from youtube
#[cfg(feature = "old_downloader")]
async fn rustDL(msg: &Message, ctx: &Context, vid: &mut VidInfo, yt: String, path: &Path) -> Result<(), Error> {

    let id = match Id::from_str(&yt) {  // Does it again, but this time its from the api
        Ok(T) => T,
        Err(_) => {errHandle(msg, ctx, 0).await; return Err(Error);},
    };

     // Starts a descrambler for the Video Data
     let descrambler = match VideoFetcher::from_id(id.into_owned()) // Fetches Videoinfo, should it exists
        .unwrap()
        .fetch()
        .await {
            Ok(T) => T,
            Err(_) => {errHandle(msg, ctx, 0).await; return Err(Error);},
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
            Err(_) => {errHandle(msg, ctx, 1).await; return Err(Error);}
        };

    // Tries to trim the video
    match editVideo(&vid).await {
        Ok(()) => println!("Successful Edit!"),
        Err(_) => {errHandle(msg, ctx, 2).await; return Err(Error);},
    };  

    Ok(())
}

#[cfg(not(feature = "old_downloader"))]
async fn updateInfo(vid: &mut VidInfo, msg: &Message) {

    // Saves some of the video info in a better format
    vid.start = matchStart(&msg.content.as_str(), &vid.v_length).await;
    vid.u_length = matchLength(&msg.content.as_str(), &vid.v_length, &vid.start).await;
    vid.u_id = msg.author.id.get().to_string().clone();

    println!("{:#?}", &vid);

} 


#[cfg(not(feature = "old_downloader"))]
async fn rustDL(msg: &Message, ctx: &Context, vid: &mut VidInfo, yt: String, path: &Path) -> Result<(), Error> {

    let form: String = format!("https://www.youtube.com/watch?v={}", yt);

    println!("{}", form);

    let video = match YoutubeDl::new(form).run() {
        Ok(T) => T,
        Err(E) => {
            println!("{}", E);
            return  Err(Error);
        },  
    };

    match video {
        youtube_dl::YoutubeDlOutput::SingleVideo(V) => {
            vid.name = match V.title {
                Some(T) => T,
                None => {
                        errHandle(msg, ctx, 0).await; 
                        return Err(Error);
                },
            };
            vid.v_length = match V.duration {
                Some(T) => T.as_i64().unwrap() as u64,
                None => {
                    errHandle(msg, ctx, 0).await; 
                    return Err(Error);
                },
            };            
        },
        _ => {
            errHandle(msg, ctx, 0).await; 
            return Err(Error);
        },
    }

    updateInfo(vid, &msg).await;


    let section: String = format!("*{}-{}", formatSec(vid.start), formatSec(vid.start + vid.u_length));

    let command: String = format!(
        "yt-dlp -o {} --download-sections {} -x --audio-format mp3 -f bestaudio https://www.youtube.com/watch?v={} --force-overwrites --force-keyframes-at-cuts",
        path.to_str().unwrap(), section, yt
    );

    println!("{}", command);

    let down_res: Result<Result<std::process::Output, std::io::Error>, task::JoinError> = task::spawn_blocking(move || {
        Command::new("nu")
            .arg("-c")
            .arg(command)
            .output()  
    }).await;

    match down_res {
        Ok(T) => {
            match T {
                Ok(_) => println!("Successful Download!"),
                Err(_) => {
                    println!("Output Error!");
                    errHandle(msg, ctx, 1).await; 
                    return Err(Error);
                },
            }
        },
        Err(_) => {
            println!("Join Error!");
            errHandle(msg, ctx, 1).await; 
            return Err(Error);
        },    
    }


    Ok(())
}




//--------------------------------------------------------------------------------------------------------------------------
// Handles most of the logic for the YouTube video detection 
async fn userMapCheckAndUpdate(msg: &Message, ctx: &Context) {

    say(msg, ctx, "Aight".to_string()).await;

    delete(msg, ctx).await;

    // Sets filepath
    let u_name: String = removeUserAt(msg.author.id.get().to_string());
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

    match rustDL(msg, ctx, &mut vid, yt, path).await {
        Ok(()) => (),
        Err(_) => return,
    }
       
    let mut response: String = SET_RESPONSE.to_string();
    response.push_str(msg.author.name.as_str());


    say(msg, ctx, response).await;

}


//--------------------------------------------------------------------------------------------------------------------------
// Trims the video file
#[cfg(feature = "old_downloader")]
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
        1 => "Unable to download Video!",
        2 => "Unable to edit video, old video has been overwritten!",
        _ => "Invalid Case! / Not Implemented!",
    };

    say(msg, ctx, err.to_string()).await;

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
            let mut len: u64 = capture.get(1)
                                      .unwrap()
                                      .as_str()
                                      .to_string()
                                      .to_owned()
                                      .clone()
                                      .parse::<u64>()
                                      .unwrap();

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
