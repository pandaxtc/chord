use iced::{widget::*, Container, Element, Scrollable, Text};

use chord::common::{ChordWidget, Message};

#[derive(Debug, Clone)]
pub struct MessageList {
    scroll_state: scrollable::State,
    messages: Vec<String>,
}

impl ChordWidget for MessageList {
    fn update(&self, _msg: Message) {
        ()
    }

    fn view(&mut self) -> Element<Message> {
        let content = Scrollable::new(&mut self.scroll_state).padding(20).push(
            self.messages
                .iter()
                .fold(Column::new().spacing(20), |column, msg| {
                    column.push(Text::new(msg.clone()))
                }),
        );

        Container::new(content).center_x().center_y().into()
    }
}

impl MessageList {
    pub fn new() -> MessageList {
        let mut message = Vec::new();
        message.push(String::from("wooo! test"));
        MessageList {
            scroll_state: scrollable::State::default(),
            messages: message,
        }
    }
}
