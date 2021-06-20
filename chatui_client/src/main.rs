use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:3000";
const MSG_SIZE: usize = 64;

fn chat(name: String) {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect.");

    client
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking.");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];

        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = String::from_utf8(
                    buff.into_iter()
                        .take_while(|&x| x != 0)
                        .collect::<Vec<u8>>(),
                )
                .expect("Invalid utf8 message.");

                let components = msg.split(": ").collect::<Vec<&str>>();

                if components[0] != name {
                    println!("{}", msg)
                };
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed.");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = format!("{}: {}", name, msg).clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to socket failed.");
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(100));
    });

    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Reading from stdin failed.");
        let msg = buff.trim().to_string();
        if msg == ":q" || tx.send(msg).is_err() {
            break;
        }
    }
    println!("Bye.");
}

fn main() {
    println!("Enter your name:");
    let mut name = String::new();

    io::stdin()
        .read_line(&mut name)
        .expect("Reading from stdin failed.");

    if !name.is_empty() {
        chat(name.trim().to_string());
    }
}
