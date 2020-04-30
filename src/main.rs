use std::process::Command;
use std::string::FromUtf8Error;

fn main() -> Result<(), FromUtf8Error> {
    let output = Command::new("ls")
        .arg("-l")
        .arg("/Users/stepher2/")
        .output()
        .unwrap()
        .stdout;

    unsafe {
        String::from_utf8_unchecked(output)
            .lines()
            .filter(|s| s.ends_with("py"))
            .for_each(|line| println!("{}", line));
    }

    Ok(())
}
