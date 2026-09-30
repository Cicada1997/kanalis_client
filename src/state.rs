use crate::protocol::{ MessageId, User, Member, Channel, ChannelId, UserDetails, ClientPacket, ServerPacket, Error };
use crate::net::ConnEvent;

use tokio::sync::mpsc;
use serde::{ Serialize, Deserialize };
use chrono::naive::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub user: User,
    pub channel_id: ChannelId,
    pub content: String,
    pub timestamp: NaiveDateTime,
}

pub struct State {
    // Chat Data
    pub channels: Vec<Channel>,
    pub members: Vec<Member>,
    pub messages: Vec<Message>,
    pub user: Option<UserDetails>,
    pub current_channel: ChannelId,
    pub current_message: String,

    // Connection
    pub sender: Option<mpsc::Sender<ClientPacket>>,

    // Auth
    pub current_token: Option<String>,
    pub auth_status: AuthStatus,
    pub username_input_field: String,
    pub password_input_field: String,
}

#[derive(Debug)]
pub enum AuthStatus {
    LoggedIn,
    Waiting,
    LoggedOut,
}

impl Default for State {
    fn default() -> Self {
        Self {
            channels: vec![],
            messages: vec![],
            members: vec![],
            user: None,
            current_channel: 0,
            current_message: String::new(),

            sender: None,

            current_token: dotenv::var("TOKEN").ok(),
            auth_status: AuthStatus::LoggedOut,
            username_input_field: String::new(),
            password_input_field: String::new(),
        }
    }
}

impl State {
    pub fn apply_conn_event(&mut self, event: ConnEvent) {
        match event {
            ConnEvent::Connecting => {
                self.auth_status = AuthStatus::Waiting;
            }
            ConnEvent::Connected(sender) => {
                self.sender = Some(sender);
            }
            ConnEvent::Disconnected => {
                println!("Disconnected from server");
                self.sender = None;
            }
            ConnEvent::Exit(reason) => {
                println!("exiting with reason: {reason:?}");
                std::process::exit(0)
            },
            ConnEvent::Packet(packet) => self.handle_server_packet(packet),
        }
    }

    fn handle_server_packet(&mut self, packet: ServerPacket) {
        match packet {
            ServerPacket::ServerData { name, channels, members } => {
                println!("Server name is {name}");
                if let Some(first_channel) = channels.first() {
                    self.current_channel = first_channel.id;
                }
                self.channels = channels;
                self.members = members;
            }
            ServerPacket::LoginSuccess { user } => {
                self.auth_status = AuthStatus::LoggedIn;
                self.user = Some(user);
            }
            ServerPacket::NewMessage { id, user, channel_id, timestamp, content } => {
                self.messages.push(Message { id, user, channel_id, content, timestamp });
            }
            ServerPacket::Error { code, reason } => {
                if matches!(code, Error::AuthFail) {
                    eprintln!("Invalid token. Restart with a working one.");
                } else {
                    eprintln!("Error: {code:?}: {reason}");
                }
            }
        }
    }

    pub fn send_current_message(&mut self) {
        if self.current_message.trim().is_empty() { return; }

        if let Some(ref mut sender) = self.sender {
            let packet = ClientPacket::Message {
                channel_id: self.current_channel,
                content: self.current_message.clone(),
            };

            match sender.try_send(packet) {
                Ok(()) => self.current_message.clear(),
                Err(e) => eprintln!("Failed to send message: {e}"),
            }
        } else {
            eprintln!("Unable to send message, sender is not yet defined.");
        }
    }

    pub fn submit_login(&mut self) {
        let username = self.username_input_field.clone();
        let hashword = self.password_input_field.clone();

        self.username_input_field.clear();
        self.password_input_field.clear();

        let client = reqwest::blocking::Client::new();
        let res = client.post("https://auth.kattmys.se/login")
            .json(&crate::protocol::kattauth::LoginDetails {
                username,
                hashword,
            })
            .send();

        match res {
            Ok(resp) => {
                dbg!(&resp.status());
                if resp.status().is_success() {
                    let token: String = match resp.json() {
                        Ok(token) => token,
                        Err(_e) => { return }
                    };
                    self.current_token = Some(token);
                }

            }
            Err(e) => {
                dbg!(&e);
            }
        }
    }
}
