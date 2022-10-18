use std::fmt::Error;
use std::sync::Arc;

use serenity::model::prelude::{ChannelId, GuildId, ChannelType, Member, GuildChannel};
use serenity::model::voice::VoiceState;
use serenity::{client::*};
use songbird::{ffmpeg, Songbird};

use crate::helper::*;

//--------------------------------------------------------------------------------------------------------------------------
// Const Declaration
pub static BOT_ID: u64 = 909567837964746863;

//--------------------------------------------------------------------------------------------------------------------------
// Joins the Voice channel and plays sound
pub async fn joinVoice(ctx: Context, old: Option<VoiceState>, new: VoiceState) -> Result<(), Error> {

    let user_id: u64 = new.user_id.0;

    let channel_id = match checkError(user_id, old, &new) {
        Ok(C) => C,
        Err(_) => return Err(Error),
    };

    // Gets id of channel
    let guild_id: GuildId = new.guild_id.unwrap();

    let guild_channels = ctx.http.get_channels(guild_id.0).await;

    match checkDuplicate(guild_channels, &ctx.cache).await {
        Ok(_) => (),
        Err(_) => return Err(Error),
    };


    // Builds path for video
    let path: String = buildVidPath(user_id.to_string());

    // Checks if path exists
    match checkVidPath(&path) {
        true => (),
        false => return Err(Error),
    }

    // Gets songbird instance
    let manager = songbird::get(&ctx).await
                                                    .expect("Unable to get songbird instance!")
                                                    .clone();

    // Joins the voice channel
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
// Disconnects Manager from Call
pub async fn removeManager(manager: &Arc<Songbird>, guild_id: GuildId) {
    
    match manager.remove(guild_id).await {
        Ok(()) => (),
        Err(E) => println!("{:?}", E),
    }

}

//--------------------------------------------------------------------------------------------------------------------------
// Checks if the bot already is in a channel
async fn checkDuplicate(guild_channels: Result<Vec<GuildChannel>, serenity::prelude::SerenityError>, cache: &Arc<Cache>) -> Result<(), Error>{

    // Loop over all channels in guild
    for c in guild_channels.unwrap() {
        
        // Continue only if Voice Channel
        if c.kind != ChannelType::Voice {
            continue;
        }

        // Get member Vector of Voice Channel
        let chan_members: Result<Vec<Member>, serenity::Error> = c.members(&cache).await;

        // Check if Vector is empty
        let members: Vec<Member> = match chan_members {
            Ok(M) => M,
            Err(_) => continue, 
        };

        // Loop over Vector
        for m in members {
            
            // Check if Bot is in a channel already, if it is an error is returned
            if m.user.id != BOT_ID {
                continue;
            }

            return Err(Error);
        }
    }

    Ok(())
}


//--------------------------------------------------------------------------------------------------------------------------
// Checks conditions for playing an intro
fn checkError(user_id: u64, old: Option<VoiceState>, new: &VoiceState) -> Result<ChannelId, Error> {

    // Checks if user joined is the bot
    if user_id == BOT_ID {
        return Err(Error);
    }

    // Checks if user wasn't in a channel before
    match old {
        Some(_) => return Err(Error),
        None => (),
    }

    // Checks if user is in new channel now and gets ID
    let channel_id: ChannelId = match new.channel_id {
        Some(S) => S,
        None => return Err(Error),
    };

    Ok(channel_id)
}

