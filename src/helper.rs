
pub fn removeUserAt(name: String) -> String {

    let ext: &str = ".mp3"; // File Extension
    let mut user_id: String = "./src/vid/".to_string(); // Path for file
    user_id.push_str(&name); // Adds user ID to filepath

    user_id = user_id.replace("@", "");  // Removes the "@" symbol from the id  
    
    user_id.push_str(ext);  // Adds extension at the end of the string

    user_id
}

pub fn buildVidPath(name: String) -> String {

    let ext: &str = ".mp3";
    let mut filepath: String = "./src/vid/".to_string();

    filepath.push_str(&name);
    filepath.push_str(ext);
    
    filepath
}