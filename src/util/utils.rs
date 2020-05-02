use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn build_table_rows(map: HashMap<String, String>) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let mut vec_of_vecs = Vec::new();

    for (key, value) in map {
        let new_vec = vec![key, value];
        vec_of_vecs.push(new_vec);
    }

    vec_of_vecs.sort();
    Ok(vec_of_vecs)
}

pub fn copy_to_clipboard(string_to_copy: &str) -> Result<(), Box<dyn Error>> {
    let process = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(Stdio::piped())
        .spawn()?
        .stdin
        .unwrap()
        .write(string_to_copy.as_bytes());

    if let Err(e) = process {
        println!("Encountered error: {}", e);
    }

    Ok(())
}
