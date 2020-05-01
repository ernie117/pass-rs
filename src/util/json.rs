use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::error::Error;

pub fn read_file() -> Result<HashMap<String, String>, Box<dyn Error>> {
    let file = File::open("src/passwords.json")?;
    let bufreader = BufReader::new(file);

    let map: HashMap<String, String> = serde_json::from_reader(bufreader)?;

    Ok(map)
}