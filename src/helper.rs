use std::{env,
          collections::HashMap,
          fs::File,
          io::{BufReader, BufRead, Lines},
          path::Path};

use serenity::{model::prelude::Message,
               prelude::Context};

use tokio::fs::create_dir_all;

use crate::predict::{UserPrediction};

//--------------------------------------------------------------------------------------------------------------------------
// Removes @ in userID
pub fn removeUserAt(name: String) -> String {

    let ext: &str = ".mp3"; // File Extension
    let mut user_id: String = "./src/vid/".to_string(); // Path for file
    user_id.push_str(&name); // Adds user ID to filepath

    user_id = user_id.replace("@", "");  // Removes the "@" symbol from the id  
    
    user_id.push_str(ext);  // Adds extension at the end of the string

    user_id
}

//--------------------------------------------------------------------------------------------------------------------------
// Checks if a filepath exists
pub fn checkVidPath(path: &String) -> bool {
    Path::new(&path).exists()
}

//--------------------------------------------------------------------------------------------------------------------------
// Creates path for txt
pub fn buildTxtPath() -> String {

    let mut current: String = env::current_dir().expect("Unable to get current directory!").to_str().unwrap().to_string();
    let filepath: &str = "\\src\\struct\\struct.txt";

    current.push_str(filepath);

    current
}

//--------------------------------------------------------------------------------------------------------------------------
// Fills user array with data
pub fn fillStruct() -> HashMap<u64, UserPrediction> {

    let mut map: HashMap<u64, UserPrediction> = HashMap::new();
    let path: String = buildTxtPath();

    // Checks if file exists, returns empty map otherwise
    let file: File = match File::open(&path) {
        Ok(F) => F,
        _ => {
            File::create(path).expect("File doesn't exist and is unable to be created!");
            println!("Created Struct File!");
            return map;
        },
    };

    let reader: BufReader<File> = BufReader::new(file);
    let lines: Lines<BufReader<File>> = reader.lines();

    for line in lines {
       if let Ok(text) = line {
            let vec: Vec<&str> = text.as_str().split("=").collect();
            if vec.len() != 4 {return map};

            let pre: UserPrediction = UserPrediction{   user_id: vec[1].parse::<u64>().expect("Couldn't parse u64"),
                                                        prediction: vec[2].parse::<String>().expect("Couldn't parse String"),
                                                        user_name: vec[3].parse::<String>().expect("Couldn't parse String ")};

            map.insert(vec[0].parse::<u64>().expect("Couldn't Parse u64"), pre);
        };
    }

    map
}

//--------------------------------------------------------------------------------------------------------------------------
// Creates path for file to edit
pub fn buildVidPath(name: String) -> String {
    
    let mut current: String = env::current_dir().expect("Unable to get current directory!").to_str().unwrap().to_string();
    let filepath: &str = "\\src\\vid\\";
    let ext: &str = ".mp3";

    current.push_str(filepath);
    current.push_str(&name);
    current.push_str(ext);
    
    current
}

//--------------------------------------------------------------------------------------------------------------------------
// Checks if directories exist and creates them if not (first time startup)
pub async fn checkDirs() {

    let current_dir: String = env::current_dir().expect("Unable to get working directory!")
                                                .to_str()
                                                .unwrap()
                                                .to_string();

    let mut structlocal: String = current_dir.clone();
    let mut vidlocal: String = current_dir.clone();

    structlocal.push_str("\\src\\struct\\");
    vidlocal.push_str("\\src\\vid\\");

    let structpath: &Path = Path::new(&structlocal);
    let vidpath: &Path = Path::new(&vidlocal);

    // Checks if the needed directories exist and creates them if not
    if !structpath.exists() {
        create_dir_all(structpath).await.expect("Unable to create folder ./src/struct/");
        println!("Created Struct Directory!");
    }
    
    if !vidpath.exists() {
        create_dir_all(vidpath).await.expect("Unable to create folder ./src/vid/");
        println!("Created Video Directory!");
    }

}


//--------------------------------------------------------------------------------------------------------------------------
// Writes Message to provided channel
pub async fn say(msg: &Message, ctx: &Context, out: String) {
    if let Err(why) = msg.channel_id.say(&ctx.http, out).await {
        println!("Send Message failed. Error: {:?}", why)
    }
}


//--------------------------------------------------------------------------------------------------------------------------
// Deletes Message in provided channel
pub async fn delete(msg: &Message, ctx: &Context) {
    if let Err(why) = msg.delete(&ctx.http).await {
        println!("Delete Message failed Error: {:?}", why)
    }
}


//--------------------------------------------------------------------------------------------------------------------------
// Gets path for the TimerFile
pub async fn findTimerPath() -> Option<String> {
    
    let current_dir: String = env::current_dir().expect("Unable to get working directory!")
                                                .to_str()
                                                .unwrap()
                                                .to_string();

    let mut timer: String = current_dir.clone();

    timer.push_str("\\src\\vid\\timer.mp3");

    let path: &Path = Path::new(&timer);

    if !path.exists() {
        println!("No timer.mp3 provided");
        return None;
    }

    Some(path.to_str().unwrap().to_string())
}

//--------------------------------------------------------------------------------------------------------------------------
// Stupid ass time formatter
pub fn formatSec(secs: u64) -> String {
    let hours: u64 = secs / 3600;
    let minutes: u64 = secs / 60;
    let seconds: u64 = secs % 60;

    format!("{}:{:02}:{:02}", hours, minutes, seconds)
} 