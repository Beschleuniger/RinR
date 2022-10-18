use std::fmt::Error;
use std::thread;
use std::time::Duration;

use regex::Regex;
use serenity::model::prelude::{Message, ChannelType, Member, GuildId};
use serenity::prelude::Context;
use songbird::ffmpeg;

use crate::helper::{say, findTimerPath};
use crate::voice::removeManager;

//--------------------------------------------------------------------------------------------------------------------------
// Parses command input and starts timer
pub async fn timer(msg: &Message, ctx: &Context) {

    let r_minutes: Regex =  Regex::new(r"minutes=([0-9]{1,3})").unwrap();   
    let r_seconds: Regex = Regex::new(r"seconds=([0-9]{1,2})").unwrap();
    let cmd: String = msg.content.clone();

    let author_id: u64 = msg.author.id.0;
    let guild_id: GuildId = msg.guild_id.unwrap();

    let v_channel_id: u64 = match getVoiceIfActive(author_id, guild_id.0, &ctx).await {
        Some(C) => Some(C).unwrap(),
        None => 0,
    };

    let u_min: u64 = match r_minutes.captures(&cmd) {   
        Some(capture) => {
            capture.get(1)
                                    .unwrap()
                                    .as_str()
                                    .to_string()
                                    .to_owned()
                                    .clone()
                                    .parse::<u64>()
                                    .unwrap()
        },  
        None => {
            0
        }
    };

    let u_sec: u64 = match r_seconds.captures(&cmd) {   
        Some(capture) => {
            let mut s: u64 = capture.get(1)
                                    .unwrap()
                                    .as_str()
                                    .to_string()
                                    .to_owned()
                                    .clone()
                                    .parse::<u64>()
                                    .unwrap();
            if s > 59 {s = 59;};
            s
        },  
        None => {
            0
        }
    };

    if u_min + u_sec == 0 {

        say(msg, ctx, "You didn't enter a valid time amount, dumbass!".to_string()).await;

        return
    }

    let timer_phrase: String = buildTimerPhrase(&u_min, &u_sec).await;

    say(msg, ctx, timer_phrase).await;

    waitTime((u_min * 60) + u_sec).await;

    match resolveVoiceChannel(ctx, guild_id, v_channel_id).await{
        Ok(_) => (),
        Err(_) => (),
    };

    let mut out: String = String::from("Your timer has ended!\n<@");
    out.push_str(&msg.author.id.0.to_string().as_str());
    out.push_str(">");

    say(msg, ctx, out).await;

} 



//--------------------------------------------------------------------------------------------------------------------------
// Joins and Leaves a Voice Channel
async fn resolveVoiceChannel(ctx: &Context, guild_id: GuildId, channel_id: u64) -> Result<(), Error> {

    let path: String = match findTimerPath().await {
        Some(C) => C,
        None => return Err(Error),
    };

    let manager = songbird::get(&ctx).await
                                                            .expect("Unable to get songbird instance!")
                                                            .clone();
                                                            
    let _handler = manager.join(guild_id, channel_id).await;

    if let Some(handler_lock) = manager.get(guild_id) {

        let source = match ffmpeg(path).await {
            Ok(I) => I,
            Err(_) => {
                removeManager(&manager, guild_id).await;
                return Err(Error);
            },
        };

        // Starts playing from source file
        let handler = handler_lock.lock().await.play_source(source);

        // Sleeps for the length of the audio file so that it can play
        if let Some(dur) = handler.metadata().duration {
            tokio::time::sleep(dur).await;
        } 
        
    } else {
        println!("Unexpected error");
        return Err(Error);
    }

    removeManager(&manager, guild_id).await;

    Ok(())
}


//--------------------------------------------------------------------------------------------------------------------------
// Builds timer phrase to say in the chat
async fn buildTimerPhrase(min: &u64, sec: &u64) -> String {
    let mut out: String = String::from("Timer set for ");
    out.push_str(min.to_string().as_str());
    out.push_str(" minute(s) and ");
    out.push_str(sec.to_string().as_str());
    out.push_str(" second(s)");

    out
}


//--------------------------------------------------------------------------------------------------------------------------
// Sleeps
async fn waitTime(time: u64) {
    thread::sleep(Duration::from_secs(time));
}


//--------------------------------------------------------------------------------------------------------------------------
// Gets Voice Channel a user is in if any
async fn getVoiceIfActive(author_id: u64, guild_id: u64, ctx: &Context) -> Option<u64> {

    for c in ctx.http.get_channels(guild_id).await.unwrap() {
        
        if c.kind != ChannelType::Voice {
            continue;
        }
        
        // Get member Vector of Voice Channel
        let chan_members: Result<Vec<Member>, serenity::Error> = c.members(&ctx).await;

        // Check if Vector is empty
        let members: Vec<Member> = match chan_members {
            Ok(M) => M,
            Err(_) => continue, 
        };

        // Loop over Vector
        for m in members {
            
            // Check if user is in a channel 
            if m.user.id != author_id {
                continue;
            } else {
                return Some(c.id.0);
            }

        }
    }
    None
}