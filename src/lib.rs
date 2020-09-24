pub mod common {
    // General IPC
    pub enum DiscordEvent {
        Heartbeat,
        MsgRecv(String),
        DiscordError(String),
    }

    // Interwidget communication B)
    #[derive(Debug, Clone)]
    pub enum IntMessage {
        MsgRecv(String),
    }
}
