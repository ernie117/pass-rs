use std::io::{Error, Write};
use std::process::{Command, Stdio};

fn main() {

    match copy_to_clipboard("Here is a string") {
        Err(e) => println!("Encountered an error: {}", e),
        Ok(_) => {}
    }

}


fn copy_to_clipboard(string_to_copy: &str) -> Result<(), Error> {
    let process = Command::new("pbcopy")
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
