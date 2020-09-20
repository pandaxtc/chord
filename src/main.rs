use std::env;
use std::sync::mpsc;
use serenity::async_trait;
use serenity::prelude;
use serenity::client::{Client, Context, EventHandler};
use serenity::model::{
    gateway::Ready, channel::Message
};
use serenity::framework::standard::{
    StandardFramework,
    macros::{
        group
    }
};
use iced::{
    Application, executor, Command, Element, Container,
    Length, Settings, Column, Text
};

struct DiscordPipe;

impl prelude::TypeMapKey for DiscordPipe {
    type Value = std::sync::Mutex<mpsc::Sender<DiscordEvent>>;
}

#[tokio::main]
pub async fn main() {
    // IPC
    let (send, _receive): (mpsc::Sender<DiscordEvent>, mpsc::Receiver<DiscordEvent>) = mpsc::channel();
    let serene_send = send.clone();

    // Spawn Discord manager
    let _serene = tokio::spawn(async move {
        let framework = StandardFramework::new()
        .group(&GENERAL_GROUP);

        // Login with a bot token from the environment
        let token = env::var("DISCORD_TOKEN").expect("token");
        let mut client = Client::new(token)
            .event_handler(Handler)
            .framework(framework)
            .await
            .expect("Error creating client");

        // Pass down send channel clone to client data.
        {
            let mut data = client.data.write().await;
            data.insert::<DiscordPipe>(std::sync::Mutex::new(serene_send));
        }

        if let Err(why) = client.start().await {
            println!("An error occurred while running the client: {:?}", why);
        }
    });

    // Run GUI
    let _gui = tokio::spawn(async move {
        Chord::run(Settings::default())
    });

    futures::join!(_serene, _gui);
}

// General IPC
enum DiscordEvent {
    MsgRecv(Message),
}

// Serene
#[group]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected to user {}#{}", ready.user.name, ready.user.discriminator);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        match ctx.data.read().await.get::<DiscordPipe>()
        .unwrap()
        .lock()
        .unwrap()
        .send(DiscordEvent::MsgRecv(msg)) {
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
        (
            Chord,
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Chord")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Container::new(
            Column::new()
                .push(
                    Text::new(String::from("Receiving messages...")),
                )
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}