use std::{collections::HashMap, env, fmt::Debug, fs::File, io::{BufRead, BufReader, Lines, Read}, path::Path, str::FromStr, sync::mpsc::Sender};

use chrono::{NaiveDate, NaiveTime, Timelike};
use serenity::{all::{ChannelId, UserId}, model::prelude::Message, prelude::{Context, TypeMapKey}};

use strum::Display;
use tokio::{fs::{create_dir_all, File as aFile}, io::AsyncWriteExt};

use serde::{Serialize, Deserialize};

use crate::predict::UserPrediction;
use crate::event::Command as RinrCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyEvent {
    pub name: String,
    pub id: u64,
    pub message: Option<String>,
    pub timestamp: NaiveTime,
    pub subscribers: Vec<UserId>,
    pub command: Option<String>,
    pub date: NaiveDate,
    pub interval: Timeslice,
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, PartialEq, Eq)]
pub enum Timeslice {
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Once,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSignal {
    pub event_type: RinrCommand,
    pub event_info: Option<DailyEvent>,
    pub channel_id: ChannelId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RinrOptions {
    pub bot_channel: Option<u64>,
    pub events: Vec<DailyEvent>,
    pub id_counter: u64,
    reserved2: u64,
    reserved3: u64,
}

pub struct DailyEventSignalKey;


impl TypeMapKey for DailyEventSignalKey {
    type Value = Sender<EventSignal>;
}

impl Default for DailyEvent {
    fn default() -> DailyEvent {
        DailyEvent { 
            name: "Default".to_string(),
            id: 0,
            message: None,
            timestamp: NaiveTime::default(),
            subscribers: vec![],
            command: None,
            date: NaiveDate::default(),
            interval: Timeslice::Once, 
        }
    }
}

impl Default for Timeslice {
    fn default() -> Timeslice {
        Timeslice::Daily
    }
}

impl FromStr for Timeslice {

    type Err = ();

    fn from_str(input: &str) -> Result<Timeslice, Self::Err> {
        match input.to_lowercase().as_str() {
            "daily" => Ok(Timeslice::Daily),
            "weekly" => Ok(Timeslice::Weekly),
            "monthly" => Ok(Timeslice::Monthly),
            "yearly" => Ok(Timeslice::Yearly),
            "once" => Ok(Timeslice::Once),
            _ => Err(()),
        }
    }
}

impl TypeMapKey for RinrOptions {
    type Value = HashMap<u64, RinrOptions>;
}

impl Default for RinrOptions {
    fn default() -> RinrOptions {
        RinrOptions {
            bot_channel: None,
            events: vec![],
            id_counter: 0,
            reserved2: 0,
            reserved3: 0,
        }
    }
}

pub trait States {
    fn insert(&mut self, event: DailyEvent);

    fn subscribe(&mut self, event_data: DailyEvent) -> bool;

    fn unsubscribe(&mut self, event_data: DailyEvent) -> bool;

    fn removeEntry(&mut self, num: u64); // by name or id

    fn setChannel(&mut self, num: u64);

    fn resortEvents(&mut self);

    fn _printElements(&self);
}

impl <'a> States for &'a mut RinrOptions {
    
    fn insert(&mut self, mut event: DailyEvent) {
        
        event.id = self.id_counter;

        if event.subscribers.first().unwrap().get() == 1u64 {
            event.subscribers.clear();
        }
        
        self.events.push(event);
        self.id_counter += 1;
    }

    fn subscribe(&mut self, event_data: DailyEvent) -> bool {
        
        for event in &mut self.events {
            if event.id == event_data.id {
                let id: &UserId = event_data.subscribers.first().unwrap();

                if !event.subscribers.contains(id) {
                    event.subscribers.push(*id);
                    return true;
                }
            }
        }

        return false;
    }

    fn unsubscribe(&mut self, event_data: DailyEvent) -> bool {

        for event in &mut self.events {
            if event.id == event_data.id {
                let id: &UserId = event_data.subscribers.first().unwrap();

                if event.subscribers.contains(id) {
                    event.subscribers.retain(|x| x != id);
                    return true;
                }
            }
        }

        return false;
    }

    fn removeEntry(&mut self, num: u64) {
        self.events.retain(|x| x.id != num);
    }

    fn setChannel(&mut self, num: u64) {
        self.bot_channel = Some(num);
    }

    fn resortEvents(&mut self) {
        let curr_time: NaiveTime = chrono::offset::Local::now().time();

        self.events.sort_by_key(|e| curr_time.timeDif(&e.timestamp));
    }

    fn _printElements(&self) {
        for x in self.events.clone() {
            println!("{:#?}", x);
        }
    }

}

pub trait Diff {
    fn timeDif(&self, other: &NaiveTime) -> i64;
}

impl Diff for NaiveTime {

    fn timeDif(&self, other: &NaiveTime) -> i64 {
        let current_secs: i64 = self.num_seconds_from_midnight() as i64;
        let target_secs: i64 = other.num_seconds_from_midnight() as i64;

        let difference: i64 = target_secs - current_secs;

        
        if difference > 0 {
            difference
        } else {
            difference + 24 * 3600
        }
    }
}






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


//--------------------------------------------------------------------------------------------------------------------------
// Write to Config
pub async fn writeConfig(data: Option<&RinrOptions>) {

    let mut path: String = env::current_dir().expect("Unable to get current directory!").to_str().unwrap().to_string();
    let folder: &str = "\\src\\config\\";
    let end: &str = "config.json";

    path.push_str(folder);

    let p: &Path = Path::new(path.as_str());

    if !p.exists() {
        create_dir_all(&path).await.expect("Unable to create folder ./src/config/");
        println!("Created Config Directory!");
    }

    path.push_str(end);

    let mut file: aFile = match aFile::create(&path).await {
        Ok(F) => F,
        _ => aFile::create(&path).await.expect("File doesn't exist and is unable to be created!"),
    };

    println!("path: {}", path);
    println!("{}", serde_json::to_string(data.unwrap()).unwrap());

    let _ = match data {
        Some(R) => file.write_all(serde_json::to_string(R).unwrap().as_bytes()).await.expect("Unable to serialize config!"),
        None => file.write_all(serde_json::to_string(&createDefaultConfig()).unwrap().as_bytes()).await.expect("Unable to serialize default config!"),
    };
    
    let _ = file.flush().await;

    println!("Wrote Config!");
}





//--------------------------------------------------------------------------------------------------------------------------
// Read Config File
pub async fn readConfig() -> RinrOptions {

    
    let mut current: String = env::current_dir().expect("Unable to get current directory!").to_str().unwrap().to_string();
    let filepath: &str = "\\src\\config\\";
    let config: &str = "config.json";

    current.push_str(filepath);

    let path: &Path = Path::new(current.as_str());

    if !path.exists() {
        create_dir_all(path).await.expect("Unable to create folder ./src/config/");
        println!("Created Config Directory!");
    }

    current.push_str(config);

    let mut file: File = match File::open(&current) {
        Ok(F) => F,
        _ => {
            File::create(&current).expect("File doesn't exist and is unable to be created!");
            return createDefaultConfig();
        },
    };


    let mut json: String = String::new();

    match file.read_to_string(&mut json) {
        Ok(_) => (),
        Err(e) => {
            println!("{}", e);
            return createDefaultConfig()
        }
    }


    serde_json::from_str(&json).unwrap_or(createDefaultConfig())
}

//--------------------------------------------------------------------------------------------------------------------------
// Create default config
fn createDefaultConfig() -> RinrOptions {
    RinrOptions::default()
}