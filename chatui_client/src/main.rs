use crossterm::{
    self, cursor,
    event::{self, poll, Event, KeyCode, KeyModifiers},
    execute, terminal,
};
use serde_json::{json, Value};
use std::{
    io::{self, ErrorKind, Read, Write},
    net::TcpStream,
    process,
    sync::mpsc::{self, TryRecvError},
    time::Duration,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Terminal,
};
use unicode_width::UnicodeWidthStr;

const LOCAL: &str = "127.0.0.1:3000";
const MSG_SIZE: usize = 256;

struct InputState {
    input: String,
    messages: Vec<Value>,
}

impl Default for InputState {
    fn default() -> InputState {
        InputState {
            input: String::new(),
            messages: Vec::new(),
        }
    }
}

fn get_name() -> String {
    println!("Enter your name:");
    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .expect("Reading from stdin failed.");
    name.trim().to_string()
}

fn parse(buff: Vec<u8>) -> Value {
    let msg = String::from_utf8(
        buff.into_iter()
            .take_while(|&x| x != 0)
            .collect::<Vec<u8>>(),
    )
    .expect("Invalid utf8 message.");
    serde_json::from_str(&msg).expect("Failed to parse data.")
}

fn exit() {
    terminal::disable_raw_mode().unwrap();
    execute!(
        io::stdout(),
        cursor::RestorePosition,
        cursor::Show,
        terminal::LeaveAlternateScreen,
    )
    .unwrap();
}

fn start_rx_loop(name: String) {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect.");

    terminal::enable_raw_mode().unwrap();

    execute!(
        io::stdout(),
        cursor::Hide,
        cursor::SavePosition,
        terminal::EnterAlternateScreen
    )
    .unwrap();

    let mut app = InputState::default();

    client
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking.");

    let (tx, rx) = mpsc::channel::<String>();

    loop {
        terminal
            .draw(|f| {
                let root = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
                    .margin(2)
                    .split(f.size());

                let input = Paragraph::new(app.input.as_ref()).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Message")
                        .style(
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Rgb(163, 190, 140)),
                        ),
                );

                f.set_cursor(root[1].x + app.input.width() as u16 + 1, root[1].y + 1);
                f.render_widget(input, root[1]);

                let messages: Vec<ListItem> = app
                    .messages
                    .iter()
                    .map(|m| {
                        let t = m.clone();
                        let name = t["name"].as_str().unwrap().to_string();
                        let message = t["message"].as_str().unwrap().to_string();
                        let content = vec![Spans::from(vec![
                            Span::styled(
                                name + ": ",
                                Style::default()
                                    .add_modifier(Modifier::BOLD)
                                    .fg(Color::Rgb(216, 222, 233)),
                            ),
                            Span::styled(
                                message,
                                Style::default()
                                    .fg(Color::Rgb(129, 161, 193))
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ])];
                        ListItem::new(content)
                    })
                    .collect();

                let limit = root[0].height - 2;

                if app.messages.len() > limit as usize {
                    app.messages.remove(0);
                }

                let messages = List::new(messages.clone()).block(
                    Block::default()
                        .title(name.clone())
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().fg(Color::Rgb(129, 161, 193))),
                );

                f.render_widget(messages, root[0]);

                let mut buff = vec![0; MSG_SIZE];

                match client.read_exact(&mut buff) {
                    Ok(_) => {
                        let data = parse(buff);
                        if data["name"] != name {
                            app.messages.push(data);
                        }
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => {}
                    Err(_) => {
                        exit();
                        println!("Server stopped responding.");
                        process::exit(1);
                    }
                }

                match rx.try_recv() {
                    Ok(msg) => {
                        let mut buff = json!({ "name": name, "message": msg })
                            .to_string()
                            .into_bytes();
                        buff.resize(MSG_SIZE, 0);
                        client.write_all(&buff).expect("Writing to socket failed.");
                    }
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => process::exit(1),
                }
            })
            .unwrap();

        if poll(Duration::from_millis(0)).unwrap() {
            match event::read().unwrap() {
                Event::Key(key) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        if let KeyCode::Char('c') = key.code {
                            exit();
                            break;
                        };
                    }
                    match key.code {
                        KeyCode::Esc => {
                            exit();
                            break;
                        }
                        KeyCode::Enter => {
                            tx.send(app.input.clone()).unwrap();
                            app.messages.push(json!({"name": name, "message": app.input.drain(..).collect::<String>()}));
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let name = get_name();

    start_rx_loop(name);

    println!("Bye.");
}
