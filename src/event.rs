
/*
    Main Event loop as producer

    event execution thread as mpsc consumer
    
    execution thread will recv_timeout (find async version) until a signal has come in and execute the corresponding event, tbd
    or if it receives a new event it will rebuild the config struct and save it to the disk, if it timeouts then the event is run and the next one is scheduled

    we will also need new functions to list, create and delete these events as well as options to subscribe to them


    the queue will use a ringbuffer, rebuilding it on every add or delete


*/
use std::{env, str::FromStr, sync::mpsc::{Receiver, Sender}, time::Duration};

use chrono::{Datelike, Local, NaiveDate, NaiveTime};
use regex::Regex;
use lazy_static::lazy_static;

use serde::{Deserialize, Serialize};
use serenity::http::Http;
use serenity::all::{ChannelId, Context, Message, UserId};
use tokio::runtime::Runtime;


use crate::helper::{
    writeConfig,
    DailyEvent, DailyEventSignalKey,
    Diff, EventSignal,
    RinrOptions, States, Timeslice
};


const EVENT:        &str = "$event ";
const CREATE:       &str = "create";
const LIST:         &str = "list";
const SUBSCRIBE:    &str = "subscribe";
const UNSUBSCRIBE:  &str = "unsubscribe";
const DELETE:       &str = "delete";
const CHANNEL:      &str = "channel";


lazy_static! {

    // Mode Selection
    static ref reg_mode: Regex = Regex::new(r"(create|list|delete|subscribe|unsubscribe|channel)").unwrap();

    // Mode Add
    static ref reg_name: Regex = Regex::new(r"name=\[(.*?)\]").unwrap();
    static ref reg_desc: Regex = Regex::new(r"description=\[(.*?)\]").unwrap();
    static ref reg_time: Regex = Regex::new(r"time=\[([01]\d|2[0-3]):([0-5]\d)\]").unwrap();
    static ref reg_sub: Regex = Regex::new(r"subscribe=\[(0|1)\]").unwrap();
    static ref reg_command: Regex = Regex::new(r"command=\[(.*?)\]").unwrap();
    static ref reg_date: Regex = Regex::new(r"date=(?P<day>\d{2})/(?P<month>\d{2})/(?P<year>\d{4})").unwrap();
    static ref reg_interval: Regex = Regex::new(r"(?i)interval=(Daily|Weekly|Monthly|Yearly|Once)").unwrap();

    // Mode Remove, Subscribe, Unsubscribe
    static ref reg_id: Regex = Regex::new(r"id=(\d+)").unwrap();

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Create,
    List,
    Delete,
    Subscribe,
    Unsubscribe,
    Channel,
    Invalid,
}

impl Command {
    fn from_str(command: &str) -> Option<Self> {
        match command {
            LIST =>         Some(Command::List),
            CREATE =>       Some(Command::Create),
            DELETE =>       Some(Command::Delete),
            CHANNEL =>      Some(Command::Channel),
            SUBSCRIBE =>    Some(Command::Subscribe),
            UNSUBSCRIBE =>  Some(Command::Unsubscribe),
            _ => None,
        }
    }
}



//--------------------------------------------------------------------------------------------------------------------------
// Event thread that listens on a channel and handles Events and EventSignals
pub fn loops(mut config: RinrOptions, recv: Receiver<EventSignal>) {
    
    let http: Http = Http::new(&env::var("TOKEN").unwrap());
    
    loop {

        (&mut config).resortEvents();

        let mut duration: i64 = 0;

        if let Some(first) = config.events.first() {
            let curr_time: NaiveTime = chrono::offset::Local::now().time();
            duration = curr_time.timeDif(&first.timestamp);
        }

        if duration == 0 {
            duration = 3600;
        }

        println!("Time until next Event/Timeout: {}s", duration);

        let data: EventSignal = match recv.recv_timeout(Duration::from_secs(duration as u64)) {
            Ok(D) => D,
            Err(_) => {
                println!("Timeout!");
                activateEvent(&mut config, &http);
                continue; 
            }
        };


        match data.event_type {
            Command::List => listEvent(&config, data.channel_id, &http), 
            Command::Delete => deleteEvent(&mut config, &http, &data),
            Command::Create => createEvent(&mut config, &http, &data),
            Command::Subscribe => subscribeEvent(&mut config, &data, &http),
            Command::Unsubscribe => unsubscribeEvent(&mut config, &data, &http),
            Command::Channel => channelEvent(&mut config, data.channel_id, &http),
            Command::Invalid => continue,
        }

    }

}


//--------------------------------------------------------------------------------------------------------------------------
// Executes an event 
fn activateEvent(mut config: &mut RinrOptions, http: &Http) {

    if config.bot_channel == None {
        return;
    }
        
    if let Some(current_event) = config.events.first() {

        let current_day: NaiveDate = Local::now().date_naive();

        let mut remove: bool = false;

        match current_event.interval {
            Timeslice::Daily => (),
            Timeslice::Weekly => {
                if !((current_day - current_event.date).num_days().abs() % 7 == 0) {
                    return;
                }
            },
            Timeslice::Monthly => {
                if current_day.month() != current_event.date.month() {
                    return;
                }
            },
            Timeslice::Yearly => {
                if current_day.year() != current_event.date.year() {
                    return;
                }
            },
            Timeslice::Once => {
                if (current_day != current_event.date) && (current_day < current_event.date) {
                    return;
                }

                remove = true;
            }
        }


        let mut subsc_string: Vec<String> = current_event.subscribers.iter().map(|id| format!("<@{}>", id.get())).collect();
        
        if subsc_string.len() == 0 {
            subsc_string.push("---".to_string());
        }

        let event_string: String = format!(
            "**{}**\n{}\n{}",
            current_event.name,
            current_event.message.clone().unwrap_or("---".to_string()),
            subsc_string.join("\n"),       
        );

        let rt: Runtime = Runtime::new().unwrap();

        rt.block_on(sendWrapper(ChannelId::new(config.bot_channel.unwrap()), http, event_string));

        if current_event.command != None {
            rt.block_on(sendWrapper(ChannelId::new(config.bot_channel.unwrap()), http, current_event.command.clone().unwrap()));
        }

        if remove {
            config.removeEntry(current_event.id);
            rt.block_on(writeConfig(Some(&config)));
        }
    }
}




//--------------------------------------------------------------------------------------------------------------------------
// Subscribe to an event
fn subscribeEvent(mut config: &mut RinrOptions, data: &EventSignal, http: &Http) {

    let ret: bool = config.subscribe(data.event_info.clone().unwrap());

    
    if ret {
        let rt: Runtime = Runtime::new().unwrap();
        rt.block_on(writeConfig(Some(&config)));
        rt.block_on(sendWrapper(data.channel_id, http, "Successfully subscribed to Event".to_string()));
    }
}

//--------------------------------------------------------------------------------------------------------------------------
// Unsubscribe from an event
fn unsubscribeEvent(mut config: &mut RinrOptions, data: &EventSignal, http: &Http) {

    let ret: bool = config.unsubscribe(data.event_info.clone().unwrap());

    if ret {
        let rt: Runtime = Runtime::new().unwrap();
        rt.block_on(writeConfig(Some(&config)));
        rt.block_on(sendWrapper(data.channel_id, http, "Successfully unsubscribed from Event".to_string()));
    }

}

//--------------------------------------------------------------------------------------------------------------------------
// Deletes an event
fn deleteEvent(mut config: &mut RinrOptions, http: &Http, data: &EventSignal) {

    config.removeEntry(data.event_info.clone().unwrap().id);

    let rt: Runtime = Runtime::new().unwrap();

    rt.block_on(writeConfig(Some(&config)));

    rt.block_on(sendWrapper(data.channel_id, http, "Successfully deleted event".to_string()));

}

//--------------------------------------------------------------------------------------------------------------------------
// Creates the events
fn createEvent(mut config: &mut RinrOptions, http: &Http, data: &EventSignal) {

    if config.bot_channel == None {
        let rt: Runtime = Runtime::new().unwrap();
        rt.block_on(sendWrapper(data.channel_id, http, "Please configure the Bot channel!\nYou can do this by using $event channel.".to_string()));
        return;
    }

    let rt: Runtime = Runtime::new().unwrap();

    let event_to_add: DailyEvent = data.event_info.clone().unwrap();

    config.insert(event_to_add);

    rt.block_on(writeConfig(Some(&config)));

    rt.block_on(sendWrapper(data.channel_id, http, "Successfully added event".to_string()));
}


//--------------------------------------------------------------------------------------------------------------------------
// Sets the bot channel
fn channelEvent(mut config: &mut RinrOptions, id: ChannelId, http: &Http) {

    config.setChannel(id.get());

    let rt: Runtime = Runtime::new().unwrap();

    rt.block_on(writeConfig(Some(&config)));

    rt.block_on(sendWrapper(id, http, "Bot Channel Configured!".to_string()));
}

//--------------------------------------------------------------------------------------------------------------------------
// Lists all available events in the bot channel
fn listEvent(config: &RinrOptions, id: ChannelId, http: &Http) {

    let rt: Runtime = Runtime::new().unwrap();

    
    if config.bot_channel == None {
        rt.block_on(sendWrapper(id, http, "Please configure the Bot channel!\nYou can do this by using $event channel.".to_string()));
        return;
    } 
        
    let mut form: String = formatEvents(&config.events);

    if form.len() == 0 {
        form = String::from("No Events Available");
    }


    rt.block_on(sendWrapper(
        ChannelId::new(
            config.bot_channel.unwrap()
        ), http,
        form
    ));

}



//--------------------------------------------------------------------------------------------------------------------------
// Helps format subscribers
fn formatEvents(events: &[DailyEvent]) -> String {
    events.iter().map(formatEvent).collect::<Vec<String>>().join("\n\n")
}

//--------------------------------------------------------------------------------------------------------------------------
// Formats event info to send into the bot channel
fn formatEvent(event: &DailyEvent) -> String {
    let mut subsc_string: Vec<String> = event.subscribers.iter().map(|id| format!("<@{}>", id.get())).collect();

    if subsc_string.len() == 0 {
        subsc_string.push("No Subscribers".to_string());
    }

    format!(
        "Event: {}\nID: {}\nMessage: {}\nTime: {}\nSubscribers: {}\nCommand: {}\nDate: {}\nInterval: {}\n\n",
        event.name,
        event.id,
        event.message.as_deref().unwrap_or("No Message"),
        event.timestamp,
        subsc_string.join(", "),
        event.command.as_deref().unwrap_or("No Command"),
        event.date,
        event.interval,
    )
}


//--------------------------------------------------------------------------------------------------------------------------
// A wrapper to send messages to a channel, given an id
async fn sendWrapper(id: ChannelId, http: &Http, out: String) {
    if let Err(why) = id.say(http, out).await {
        println!("Send Message failed. Error: {:?}", why);
    }
}



//--------------------------------------------------------------------------------------------------------------------------
// Matches the command and sends an EventSignal to the Event Thread
pub async fn eventHandler(msg: &Message, ctx: &Context) {

    let mut u_data = ctx.data.write().await;

    let send: &mut Sender<EventSignal> = match u_data.get_mut::<DailyEventSignalKey>() {
        Some(D) => D,
        None => return,
    };

    let stripped_command: String = msg.content.clone()
                                        .strip_prefix(EVENT)
                                        .expect("Command got corrupted inside the program!")
                                        .to_string();


    if let Some(some_mode) = reg_mode.captures(&stripped_command) {
        if let Some(selected_mode) = Command::from_str(some_mode.get(1).unwrap().as_str()) {
            let event: EventSignal = match selected_mode {
                Command::Create => createInsert(&msg).await,
                Command::List => createListEvent(&msg).await,
                Command::Delete => createDeleteEvent(&msg).await,
                Command::Subscribe => createSubscribeEvent(&msg).await,
                Command::Unsubscribe => createUnsubscribeEvent(&msg).await,
                Command::Channel => createChannelEvent(&msg).await,
                Command::Invalid => createInvalid().await,
            };

            let _ = send.send(event);
        }
    }
}


//--------------------------------------------------------------------------------------------------------------------------
// Creates the EventSignal for subscribing to an event
async fn createSubscribeEvent(msg: &Message) -> EventSignal {

    if let Some(event_id) = reg_id.captures(&msg.content).and_then(|caps| caps.get(1).map(|m| m.as_str().parse::<u64>())) {
        if let Ok(num) = event_id {
            return EventSignal {
                event_type: Command::Subscribe,
                event_info: Some(DailyEvent { 
                    name: "Subscribe".to_string(),
                    id: num,
                    message: None,
                    timestamp: NaiveTime::default(),
                    subscribers: vec![UserId::new(msg.author.id.get())],
                    command: None,
                    date: NaiveDate::default(),
                    interval: Timeslice::default(),
                }),
                channel_id: msg.channel_id,
            }
        }
    }

    createInvalid().await
}


//--------------------------------------------------------------------------------------------------------------------------
// Creates the EventSignal for unsubscribing from an event
async fn createUnsubscribeEvent(msg: &Message) -> EventSignal {
    
    if let Some(event_id) = reg_id.captures(&msg.content).and_then(|caps| caps.get(1).map(|m| m.as_str().parse::<u64>())) {
        if let Ok(num) = event_id {
            return EventSignal {
                event_type: Command::Unsubscribe,
                event_info: Some(DailyEvent { 
                    name: "Unsubscribe".to_string(),
                    id: num,
                    message: None,
                    timestamp: NaiveTime::default(),
                    subscribers: vec![UserId::new(msg.author.id.get())],
                    command: None,
                    date: NaiveDate::default(),
                    interval: Timeslice::default(),
                }),
                channel_id: msg.channel_id,
            }
        }
    }

    createInvalid().await
}


//--------------------------------------------------------------------------------------------------------------------------
// Creates the EventSignal for event deletion
async fn createDeleteEvent(msg: &Message) -> EventSignal {
   
    if let Some(event_id) = reg_id.captures(&msg.content).and_then(|caps| caps.get(1).map(|m| m.as_str().parse::<u64>())) {
        if let Ok(num) = event_id {
            return EventSignal { 
                event_type: Command::Delete,
                event_info: Some(DailyEvent {
                    name: "Delete".to_string(),
                    id: num,
                    message: None,
                    timestamp: NaiveTime::default(),
                    subscribers: vec![],
                    command: None,
                    date: NaiveDate::default(),
                    interval: Timeslice::default(),
                }),
                channel_id: msg.channel_id,
            }
        }
    }


    createInvalid().await
}

//--------------------------------------------------------------------------------------------------------------------------
// Creates the EventSignal to create events
async fn createInsert(msg: &Message) -> EventSignal {

    let text: &String = &msg.content;

    let mut event_name: Option<String> = None;
    let mut event_description: Option<String> = None;
    let mut event_time: Option<NaiveTime> = None;
    let mut event_subscribe: u64 = 1;
    let mut event_command: Option<String> = None;
    let mut event_date: Option<NaiveDate> = None;
    let mut event_interval: Option<Timeslice> = None;

    if let Some(name) = parse_first_match(text, &reg_name) {
        println!("Name: {}", name);
        event_name = Some(name);
    }

    if let Some(description) = parse_first_match(text, &reg_desc) {
        println!("Description: {}", description);
        event_description = Some(description);
    }

    if let Some(time) = parse_time(text) {
        event_time = match time {
            Some(T) => {
                println!("Time: {}", T);
                Some(T)
            },
            None => None,
        }
    }

    if let Some(subscribe) = parse_subscribe(text) {
        println!("Subscribe: {}", subscribe);
        event_subscribe = match Some(subscribe).unwrap() {
            true => msg.author.id.get(),
            false => 1,
        }
    }

    if let Some(command) = parse_first_match(text, &reg_command) {
        println!("Command: {}", command);
        event_command = Some(command);
    }

    if let Some(date) = parse_date(text) {
        println!("Date: {}", date);
        event_date = Some(date);
        if event_date.unwrap() < Local::now().date_naive() {
            event_date = Some(Local::now().date_naive());
        }
    }

    if let Some(interval) = parse_interval(text) {
        println!("Interval: {}", interval);
        event_interval = Some(interval);
    }



    if event_name.is_none() || event_time.is_none() {
        println!("Invalid Event");
        return createInvalid().await;
    }

    if event_date == None {
        event_date = Some(Local::now().date_naive());
    }

    if event_interval == None {
        event_interval = Some(Timeslice::default());
    }



    let user: UserId = UserId::new(event_subscribe);
    let v: Vec<UserId> = vec![user];

    return EventSignal {
        event_type: Command::Create, 
        event_info: Some(DailyEvent {
            name: event_name.unwrap(),
            id: 0,
            message: event_description,
            subscribers: v,
            timestamp: event_time.unwrap(),
            command: event_command,
            date: event_date.unwrap(),
            interval: event_interval.unwrap(),
        }),
        channel_id: msg.channel_id,
    };
}


//--------------------------------------------------------------------------------------------------------------------------
// Creates an invalid event signal as a fallback
async fn createInvalid() -> EventSignal {
    EventSignal {event_type: Command::Invalid, event_info: None, channel_id: ChannelId::new(1)}
} 


//--------------------------------------------------------------------------------------------------------------------------
// Regex parsing function for the date
fn parse_date(text: &str) -> Option<NaiveDate> {
    
    let mut d: u32 = 0;
    let mut m: u32 = 0;
    let mut y: i32 = 0;
    
    if let Some(dates) = reg_date.captures(text) {
        
        if let Some(day) = dates.name("day") {
            d = day.as_str().parse::<u32>().unwrap();
        }

        if let Some(month) = dates.name("month") {
            m = month.as_str().parse::<u32>().unwrap();
        }

        if let Some(year) = dates.name("year") {
            y = year.as_str().parse::<i32>().unwrap();
        }

        return NaiveDate::from_ymd_opt(y, m, d);
    }

    None
}

//--------------------------------------------------------------------------------------------------------------------------
// Regex parsing function for the date
fn parse_interval(text: &str) -> Option<Timeslice> {
    reg_interval.captures(text).and_then(|caps| {
        let time = caps.get(1).map(|t| t.as_str());
        match time {
            Some(t) => {
                match Timeslice::from_str(t) {
                    Ok(f) => Some(f),
                    Err(_) => None,
                } 
            }
            None => None,
        }
    })
}




//--------------------------------------------------------------------------------------------------------------------------
// Regex parsing function for the subscribers
fn parse_subscribe(text: &str) -> Option<bool> {
    reg_sub.captures(text).and_then(|caps| caps.get(1).map(|m| m.as_str() == "1"))
}

//--------------------------------------------------------------------------------------------------------------------------
// Regex parsing function for the time
fn parse_time(text: &str) -> Option<Option<NaiveTime>> {
    reg_time.captures(text).and_then(|caps| {
        let hours = caps.get(1).map(|m| m.as_str().parse::<u32>().ok());
        let minutes = caps.get(2).map(|m| m.as_str().parse::<u32>().ok());
        match (hours, minutes) {
            (Some(Some(h)), Some(Some(m))) => Some(NaiveTime::from_hms_opt(h, m, 0)),
            _ => None,
        }
    })
}

//--------------------------------------------------------------------------------------------------------------------------
// Regex parsing function for the first match of strings
fn parse_first_match(text: &str, regex: &Regex) -> Option<String> {
    regex.captures(text).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}


//--------------------------------------------------------------------------------------------------------------------------
// Creates the EventSignal to set the bot channel
async fn createChannelEvent(msg: &Message) -> EventSignal {
    EventSignal {event_type: Command::Channel, event_info: None, channel_id: msg.channel_id}
}

//--------------------------------------------------------------------------------------------------------------------------
// Creates the EventSignal to List all Events
async fn createListEvent(msg: &Message) -> EventSignal {
    EventSignal {event_type: Command::List, event_info: None, channel_id: msg.channel_id}
}