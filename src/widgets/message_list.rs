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

    pub fn update(&mut self, msg: IntMessage) {
        println!("hello!");
    }

    pub fn view(&mut self) -> Element<IntMessage> {
        let mut content: Column<Text> = self
            .messages
            .iter()
            .fold(Column::new().spacing(20), |column, msg| {
                column.push(Text::new(msg.clone()))
            });

        let mut scrolling = Scrollable::new(&mut self.scroll_state)
            .padding(20)
            .push(content);

        // TODO: why doesn't this build?
        Container::new(scrolling).into()
    }
}
