pub mod common {
    pub trait ChordWidget {
        fn update(&self, msg: Message);
        fn view(&mut self) -> iced::Element<Message>;
    }

    // General IPC
    pub enum DiscordEvent {
        Heartbeat,
        MsgRecv(String),
        DiscordError(String),
    }

    // Interwidget communication B)
    #[derive(Debug, Clone)]
    pub enum Message {
        MsgRecv(String),
    }
}
