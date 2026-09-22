pub mod state;
pub mod protocol;
pub mod ui;
pub mod net;

use ui::{ update, view };
use net::{ connect };
use state::{ State };

use iced::Theme;

fn main() -> iced::Result {
    iced::application(State::default, update, view)
        .subscription(|_: &State| connect())
        .theme(|_: &State| Theme::Dark)
        .centered()
        .run()
}
