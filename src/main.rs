use std::env;
use std::fs;
use std::fs::DirEntry;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4762").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn cd(destination: &str) -> String {
    let path = Path::new(destination);
    env::set_current_dir(path).unwrap();
    pwd()
}

fn ls() -> String {
    let mut files: Vec<String> = Vec::new();
    let mut paths: Vec<DirEntry> = fs::read_dir(pwd()).unwrap().map(|r| r.unwrap()).collect();
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

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let mut request: &str = &String::from_utf8(buffer.to_vec()).unwrap();
    request = request.trim();
    let trim_chars = [' ', '\n', '\0'];
    request = request.trim_matches(&trim_chars[..]);
    let mut command_iter = request.split_whitespace();
    let command = command_iter.next().unwrap();
    let response: String = match command {
        "cd" => cd(command_iter.next().unwrap()),
        "ls" => ls(),
        "ping" => String::from("pong"),
        "pwd" => pwd(),
        "shell" => shell(command_iter.next().unwrap(), command_iter),
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
