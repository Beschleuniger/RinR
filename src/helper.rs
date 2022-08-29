use std::{env, collections::HashMap, fs::File, io::{BufReader, BufRead, Lines}, fmt::Error};

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
// Creates path for txt
pub fn buildTxtPath() -> String {

    let mut current: String = env::current_dir().expect("Unable to get current directory!").to_str().unwrap().to_string();
    let filepath: &str = "\\src\\struct\\struct.txt";

    current.push_str(filepath);

    current
}

//--------------------------------------------------------------------------------------------------------------------------
// Fills user array with data
pub fn fillStruct() -> Result<HashMap<u64, String>, Error> {

    let mut map: HashMap<u64, String> = HashMap::new();
    let path: String = buildTxtPath();

    let file: File = File::open(path).expect("Couldn't open File!");
    let reader: BufReader<File> = BufReader::new(file);
    let lines: Lines<BufReader<File>> = reader.lines();

    for line in lines {
       if let Ok(text) = line {
            let vec: Vec<&str> = text.as_str().split("=").collect();

            map.insert(vec[0].parse::<u64>().expect("Couldn't Parse u64")
                        , vec[1].parse::<String>().expect("Couldn't parse String"));
        };
    }

    println!("Saved Data: {:?}", map);
    Ok(map)
}