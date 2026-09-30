use crate::protocol::{ MessageId, User, Member, Channel, ChannelId, UserDetails, ClientPacket };

use iced::futures::channel::mpsc;

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

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Channel {
//     pub id: ChannelId,
//     pub name: String,
// }

pub struct State {
    pub channels: Vec<Channel>,
    pub members: Vec<Member>,
    pub messages: Vec<Message>,
    pub user: Option<UserDetails>,
    pub current_channel: ChannelId,
    pub sender: Option<mpsc::Sender<ClientPacket>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            channels: vec![],
            messages: vec![],
            members: vec![],
            user: None,
            current_channel: 0,
            sender: None,
        }
    }
}
