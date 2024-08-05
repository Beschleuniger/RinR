use std::collections::HashMap;
use std::io::Write;
use std::{fs, fs::File};

use serenity::model::prelude::Message;
use serenity::prelude::Context;

use crate::command::{PREDICTION, User};
use crate::helper::{buildTxtPath, say};


//--------------------------------------------------------------------------------------------------------------------------
// Struct Declaration 

#[derive(Debug, Clone)]
pub struct UserPrediction {
    pub user_id: u64,
    pub user_name: String,
    pub prediction: String,
}


//--------------------------------------------------------------------------------------------------------------------------
// Matches the subversions of the prediction command
pub async fn addPrediction(msg: &Message, ctx: &Context) {
    
    let content: String = msg.content.clone();
    let mut cmd: i8 = 0;

    if content.contains(" -list") {
        cmd = 1;
    } else if content.contains(" -add") {
        cmd = 2;
    } else if content.contains(" -remove") {
        cmd = 3;
    }

    match cmd {
        1 => printList(&msg, &ctx).await,
        2 => insertUser(&msg, &ctx).await,
        3 => removePrediction(&msg, &ctx).await, // Require admin permissions
        _ => (),
    }

}


//--------------------------------------------------------------------------------------------------------------------------
// Removes Prediction at provided ID
async fn removePrediction(msg: &Message, ctx: &Context) {
    let mut u_data = ctx.data.write().await;                  // Waits for Lock Queue on write command and then proceeds with execution 
    let u_map: &mut HashMap<u64, UserPrediction> = u_data.get_mut::<User>().unwrap();    // Gets mutable reference to the data and stores it in counter

    let content: String = checkMessageValid(&msg).await;
    if content.is_empty() {
        return;
    }

    let m_id: u64 = content.parse::<u64>().expect("Unable to parse u64");

    u_map.remove(&m_id);

    writeToFile(u_map);
}


//--------------------------------------------------------------------------------------------------------------------------
// Checks if a message is not empty to add as a prediction
async fn checkMessageValid(msg: &Message) -> String {

    let result: String = msg.content.clone()
                                    .replace(PREDICTION, "")
                                    .replace("-add ", "")
                                    .replace("-remove ", "");
    result
}


//--------------------------------------------------------------------------------------------------------------------------
// Prints all current predictions as a discord message
async fn printList(msg: &Message, ctx: &Context) {
    let mut out: String = String::new();

    let mut u_data = ctx.data.write().await;                  // Waits for Lock Queue on write command and then proceeds with execution 
    let u_map: &mut HashMap<u64, UserPrediction> = u_data.get_mut::<User>().unwrap();    // Gets mutable reference to the data and stores it in counter

    if u_map.is_empty() {out.push_str("No Predictions at the moment!")} 
    else {out.push_str("Current Predictions: \n")} 

    for (key, value) in u_map.iter() {
        out.push_str(&value.prediction);
        out.push_str(" by ");
        out.push_str(&value.user_name);
        out.push_str(", ID:");
        out.push_str(key.to_string().as_str());
        out.push_str("\n");
    }

    say(msg, ctx, out).await;
}


//--------------------------------------------------------------------------------------------------------------------------
// Adds user to user struct
async fn insertUser(msg: &Message, ctx: &Context) {  

    let prediction: String = checkMessageValid(&msg).await;
    if prediction.is_empty() {
        return;
    }

    let mut u_data = ctx.data.write().await;                  // Waits for Lock Queue on write command and then proceeds with execution 
    let u_map: &mut HashMap<u64, UserPrediction> = u_data.get_mut::<User>().unwrap();    // Gets mutable reference to the data and stores it in counter
    
    let mut highest_id: u64 = 0;
    // Checks for highest id already existing
    for (id, _pred) in u_map.iter() {
        if *id > highest_id {highest_id = *id}
    }

    let key: u64 = highest_id + 1;
    let p_struct: UserPrediction = UserPrediction {user_id: msg.author.id.get(),
                                                    prediction: prediction,
                                                    user_name: msg.author.name.clone()};

    
    u_map.insert(key, p_struct);                             // Inserts Element into Map
    println!("Full Map: {:#?}", u_map);

    writeToFile(u_map);

    say(msg, ctx, "Added Prediction".to_string()).await;

}


//--------------------------------------------------------------------------------------------------------------------------
// Writes to File
fn writeToFile(u_map: &mut HashMap<u64, UserPrediction>) {
    
    let filepath: String = buildTxtPath();

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
        test.push_str(user.1.user_id.to_string().as_str());
        test.push_str("=");
        test.push_str(user.1.prediction.as_str());
        test.push_str("=");
        test.push_str(user.1.user_name.as_str());
        test.push_str("\n");

        file.write_all(test.as_bytes()).expect("Couldn't write to File!");

    }
}