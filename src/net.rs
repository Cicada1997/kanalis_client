use crate::protocol::{ ClientPacket, ServerPacket };
use crate::ui;

use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::Stream;
use iced::stream;
use iced::Subscription;

#[derive(Clone)]
pub enum ExitType {
    SocketClosed,
    UndefinedError(String),
}

#[derive(Clone)]
pub enum ConnEvent {
    Connected(mpsc::Sender<ClientPacket>),
    Disconnected,
    Packet(ServerPacket),
    Exit(ExitType),
}


pub fn connect() -> Subscription<ui::Message> {
    Subscription::run(connect_stream).map(|conn_event| {
        ui::Message::ConnMessage(conn_event)
    })
}

pub async fn send_packet(writer: &mut OwnedWriteHalf, packet: &ClientPacket) {
    if let Ok(json_str) = serde_json::to_string(packet)
        .inspect_err(|e| eprintln!("tried to send malformed ClientPacket: {packet:?}")) {
        writer.write_all((json_str + "\n").as_bytes())
            .await
            .inspect_err(|e| eprintln!("Error in writer: {e:?}"));
    }
}

pub fn connect_stream() -> impl Stream<Item = ConnEvent> {
    stream::channel(100, async move |mut output| {
        let addr = "127.0.0.1:9090";
        let token = "1:TCiSl0xNp6sga3XoYOL1ooLA00VLli8s";

        println!("Establishing connection...");
        let (reader, mut writer) = TcpStream::connect(addr)
            .await
            .map(|r| dbg!(r))
            .expect("Could not instantiate socket.")
            .into_split();
        println!("successfully connected to host.");

        let (sender, mut receiver) = mpsc::channel::<ClientPacket>(100);

        output.send(ConnEvent::Connected(sender)).await;

        let mut reader = BufReader::new(reader).lines();

        // sending token
        send_packet(&mut writer, &ClientPacket::AuthToken(String::from(token))).await;

        loop {
            tokio::select! {
                json_str = reader.next_line() => {
                    let json_str = match json_str {
                        Ok(Some(str)) => str,
                        Ok(None) => {
                            output.send(ConnEvent::Exit(ExitType::SocketClosed)).await;
                            break;
                        }
                        Err(e) => {
                            eprintln!("Socket read error, closing connection: {e}");
                            output.send(ConnEvent::Exit(ExitType::UndefinedError(e.to_string()))).await;
                            break;
                        }
                    };

                    let packet: ServerPacket = match serde_json::from_str(&json_str) {
                        Ok(packet) => packet,
                        Err(e) => {
                            eprintln!("bad json from client: \n{json_str}\n");
                            continue;
                        }
                    };

                    output.send(
                        ConnEvent::Packet(packet)
                    ).await
                        .inspect_err(|e| eprintln!("unable to forward string {json_str:?}: {e}"));
                }

                packet = receiver.recv() => {
                    let packet = packet.unwrap();
                    // let packet = match packet {
                    //     Ok(packet) => packet,
                    //     Err(e) => {
                    //         eprintln!("{e:?}");
                    //         break;
                    //     }
                    // };

                    if let Ok(json_str) = serde_json::to_string(&packet)
                        .inspect_err(|e| eprintln!("tried to send malformed ClientPacket: {packet:?}")) {
                        writer.write_all((json_str + "\n").as_bytes())
                            .await
                            .inspect_err(|e| eprintln!("Error in writer: {e:?}"));
                    }
                }

            }
        }

        // TODO: ensure connection is closed
        // let _ = channel.send(None);
    })
}
