use std::cmp::Ordering;
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
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn cwd() -> String {
    env::current_dir().unwrap().to_string_lossy().to_string()
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
        "cd" => {
            let path = Path::new(command_iter.next().unwrap());
            env::set_current_dir(path).unwrap();
            cwd()
        }
        "ls" => {
            let mut files: Vec<String> = Vec::new();
            let mut paths: Vec<DirEntry> =
                fs::read_dir(cwd()).unwrap().map(|r| r.unwrap()).collect();
            paths.sort_by(|a, b| {
                let a_is_dir = a.file_type().unwrap().is_dir();
                let b_is_dir = b.file_type().unwrap().is_dir();
                if a_is_dir && !b_is_dir {
                    Ordering::Less
                } else if !a_is_dir && b_is_dir {
                    Ordering::Greater
                } else {
                    a.file_name().cmp(&b.file_name())
                }
            });
            for file in paths {
                files.push(file.file_name().into_string().unwrap());
            }
            files.join("\n")
        }
        "ping" => String::from("pong"),
        "pwd" => cwd(),
        "shell" => String::from_utf8(
            Command::new(command_iter.next().unwrap())
                .args(command_iter)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap(),
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
