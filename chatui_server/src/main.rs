use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

const LOCAL: &str = "127.0.0.1:3000";
const MSG_SIZE: usize = 256;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind.");

    println!("Listening on {}", LOCAL);

    server
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking.");

    let mut clients = vec![];

    let (tx, rx) = mpsc::channel::<String>();

    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected.", addr);

            let tx = tx.clone();

            clients.push(socket.try_clone().expect("Failed to clone client."));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = String::from_utf8(
                            buff.into_iter()
                                .take_while(|&x| x != 0)
                                .collect::<Vec<u8>>(),
                        )
                        .expect("Invalid utf8 message.");
                        println!("{}", msg);
                        tx.send(msg).expect("Failed to send msg to rx.");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => {}
                    Err(e) => {
                        println!("Closing connection with: {}\nDue to {}", addr, e);
                        break;
                    }
                }
                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<TcpStream>>();
        }
        sleep();
    }
}
