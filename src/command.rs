use std::{fs::{self, DirEntry}, path::Path, env, error::Error, process::Command, net::TcpStream, time::Duration, io::{Read, Write}};

use telnet::{Telnet, Event};

pub fn cat(path: &str) -> String {
    let path = Path::new(path);
    String::from_utf8(fs::read(path).unwrap()).unwrap()
}

pub fn cd(destination: &str) -> Result<String, std::io::Error> {
    // TODO: Change path per-worker instead of globally
    let path = Path::new(destination);
    match env::set_current_dir(path) {
        Ok(_) => Ok(pwd()),
        Err(e) => Err(e),
    }
}

pub fn get(url: &str, destination: &str) -> Result<(), Box<dyn Error>> {
    let response = minreq::get(url).send()?;
    let path = Path::new(destination);
    match fs::write(path, response.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn ls(path: &str) -> String {
    let mut files: Vec<String> = Vec::new();
    let mut paths: Vec<DirEntry> = fs::read_dir(path).unwrap().map(|r| r.unwrap()).collect();
    paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    for file in paths {
        files.push(file.file_name().into_string().unwrap());
    }
    files.join("\n")
}

pub fn pwd() -> String {
    env::current_dir().unwrap().to_string_lossy().to_string()
}

pub fn shell<'a, T>(command: &str, args: T) -> String
where
    T: Iterator<Item = &'a str>,
{
    String::from_utf8(Command::new(command).args(args).output().unwrap().stdout).unwrap()
}

pub fn telnet(stream: &mut TcpStream, host: &str, port: &u16) -> Result<String, std::io::Error> {
    let mut telnet = Telnet::connect((host, *port), 4096)?;
    let mut stream_buffer = [0; 4096];
    stream.set_read_timeout(Some(Duration::new(1, 0)))?;
    'main: loop {
        match stream.read(&mut stream_buffer) {
            Ok(read) => {
                match String::from_utf8_lossy(&stream_buffer[0..read])
                    .as_ref()
                    .trim()
                {
                    "quit" => break 'main,
                    _ => telnet.write(&stream_buffer[0..read]).unwrap(),
                };
            }
            Err(_) => {}
        };
        match telnet.read_timeout(Duration::new(1, 0)) {
            Ok(event) => {
                if let Event::Data(buffer) = event {
                    stream.write(&buffer).unwrap();
                }
            }
            Err(e) => {
                println!("telnet read error: {:?}\n", e);
                break 'main;
            }
        }
    }
    Ok(String::from("Telnet connection closed"))
}