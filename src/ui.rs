use crate::state::{ self, State };
use crate::protocol::{ ChannelId, ServerPacket };
use crate::net::ConnEvent;

use iced::widget::{ text, text_input, button, column, Column, row };
use iced::{ Fill, Length, Element };

#[derive(Clone)]
pub enum Message {
    NewMessage(state::Message),
    SwitchChannel(ChannelId),
    ConnMessage(ConnEvent),
}

pub fn handle_conn_event(state: &mut State, event: ConnEvent) {
    match event {
        ConnEvent::Connected(sender) => state.sender = Some(sender),
        ConnEvent::Exit(reason) => {
            std::process::exit(0);
        }
        ConnEvent::Packet(packet) => {
            match packet {
                ServerPacket::ServerData { name, channels, members } => {
                    println!("Server name is {name}");
                    state.channels = channels;
                    state.members = members;
                }
                ServerPacket::NewMessage { id, user, channel_id, timestamp, content } => {
                    state.messages.push(
                        state::Message { id, user, channel_id, timestamp, content }
                    );
                }

                _ => {}
            }
        }

        _ => {}
    }
}

pub fn update(state: &mut State, event: Message) {
    match event {
        Message::ConnMessage(conn_event) => handle_conn_event(state, conn_event),

        Message::SwitchChannel(id) => {
            state.current_channel = id;
        }
        Message::NewMessage(_msg) => {}

        _ => {}
    }
}

pub fn view(state: &State) -> Element<'_, Message> {
    let mut channel_list = column![];
    for channel in state.channels.iter() {
        let ch = button(text(&channel.name))
            .on_press(Message::SwitchChannel(channel.id))
            .width(Fill);
        
        channel_list = channel_list.push(ch);
    }

    let mut message_history = column![];
    for message in state.messages.iter() {
        message_history = message_history.push(column![
            text(message.id),
            text(&message.user.name),
            text(&message.content),
        ]);
    }

    return row![
        channel_list
            .width(140),
        message_history
            .width(Length::FillPortion(1))
            .spacing(20),
    ]
        .spacing(20)
        .into()
}

