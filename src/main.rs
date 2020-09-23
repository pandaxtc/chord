use ansi_term::Color;
use iced::{executor, Application, Column, Command, Container, Element, Length, Settings, Text};
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{macros::group, StandardFramework},
    model::{channel::Message, gateway::Ready},
    prelude,
};
use std::{env, process::exit, sync::mpsc, thread, time::Duration};
use tokio::runtime::Runtime;

fn main() {
    let (tx, rx): (mpsc::Sender<DiscordEvent>, mpsc::Receiver<DiscordEvent>) = mpsc::channel();

    // Spawn Discord
    let _discord = thread::spawn(move || {
        let mut toki = Runtime::new().unwrap();

        toki.block_on(async move {
            let framework = StandardFramework::new().group(&GENERAL_GROUP);

            // Login with a bot token from the environment
            let token = env::var("DISCORD_TOKEN").expect("Discord token is not set!");
            let mut client = Client::new(token)
                .event_handler(Handler)
                .framework(framework)
                .await
                .expect("Error creating client");

            // Pass down send channel clone to client data.
            let discord_tx = tx.clone();
            {
                let mut data = client.data.write().await;
                data.insert::<DiscordPipe>(std::sync::Mutex::new(discord_tx));
            }

            if let Err(e) = tx.send(DiscordEvent::Heartbeat) {
                tx.send(DiscordEvent::DiscordError(String::from(format!("{}", e))))
                    .unwrap_or_default();
            }

            if let Err(e) = client.start().await {
                tx.send(DiscordEvent::DiscordError(String::from(format!("{}", e))))
                    .unwrap_or_default();
            }
        });
    });

    // heartbeat
    println!("Waiting for heartbeat message...");
    if let Err(e) = rx.recv_timeout(Duration::from_secs(5)) {
        println!("{}", Color::Red.paint(format!("Failed to start: {}", e)));
        exit(1);
    }

    // run chord
    Chord::run(Settings::default())
}

struct DiscordPipe;

impl prelude::TypeMapKey for DiscordPipe {
    type Value = std::sync::Mutex<mpsc::Sender<DiscordEvent>>;
}

// General IPC
enum DiscordEvent {
    Heartbeat,
    MsgRecv(String),
    DiscordError(String),
}

// Serene
#[group]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!(
            "Connected to user {}#{}",
            ready.user.name, ready.user.discriminator
        );
    }

    async fn message(&self, ctx: Context, msg: Message) {
        match ctx
            .data
            .read()
            .await
            .get::<DiscordPipe>()
            .unwrap()
            .lock()
            .unwrap()
            .send(DiscordEvent::MsgRecv(msg.content))
        {
            Ok(_) => println!("Sent message through channel"),
            Err(e) => println!("{}", e),
        }
    }
}

// Application container
struct Chord;

impl Application for Chord {
    type Executor = executor::Null;
    type Message = ();
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Chord, Command::none())
    }

    fn title(&self) -> String {
        String::from("Chord")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Container::new(Column::new().push(Text::new(String::from("Receiving messages..."))))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}
