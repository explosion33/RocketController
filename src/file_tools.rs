use std::{fs, collections::HashMap};

pub fn parse_ini(path: &str) -> Result<HashMap<String, String>, &str>{
    let contents = match fs::read_to_string(path) {
        Ok(n) => {n},
        Err(_) => {return Err("could not read file")},
    };
    

    let mut data: HashMap<String, String> = HashMap::new();

    for line in contents.split("\n") {
        let vals: Vec<&str> = line.split("=").collect();

        if vals.len() == 2 {
            data.insert(
                vals[0].trim().to_string(),
                vals[1].trim().to_string()
            );
        }
    }
    return Ok(data);
}