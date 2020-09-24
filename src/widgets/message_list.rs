use iced::{widget::*, Container, Element, Scrollable, Text};

use chord::common::IntMessage;

#[derive(Debug, Clone)]
pub struct MessageList {
    scroll_state: scrollable::State,
    messages: Vec<String>,
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

    pub fn view(&mut self) -> Element<IntMessage> {
        let content = self
            .messages
            .iter()
            .fold(Column::new().spacing(20), |column, msg| {
                column.push(Text::new(msg.clone()))
            });

        let scrolling = Scrollable::new(&mut self.scroll_state)
            .padding(20)
            .push(content);

        Container::new(scrolling).into()
    }
}
