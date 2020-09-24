use ansi_term::Color;
use iced::{executor, Application, Column, Command, Container, Element, Length, Settings, Text};
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{macros::group, StandardFramework},
    model::{channel::Message, gateway::Ready},
    prelude,
};
use std::{env, process::exit, thread, time::Duration};
use tokio::{runtime::Runtime, sync::mpsc, time::timeout};

use chord::common::{DiscordEvent, IntMessage};

mod widgets {
    pub mod message_list;
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    // TODO: figure out why these must be declared as mutable. does await mutate...?
    let (mut tx, mut rx): (mpsc::Sender<DiscordEvent>, mpsc::Receiver<DiscordEvent>) =
        mpsc::channel(100);

    // Spawn Discord
    // TODO: move this into the application, probably
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
                data.insert::<DiscordPipe>(discord_tx);
            }

            if let Err(e) = tx.send(DiscordEvent::Heartbeat).await {
                tx.send(DiscordEvent::DiscordError(String::from(format!("{}", e))))
                    .await
                    .unwrap_or_default();
            }

            if let Err(e) = client.start().await {
                tx.send(DiscordEvent::DiscordError(String::from(format!("{}", e))))
                    .await
                    .unwrap_or_default();
            }
        });
    });

    // heartbeat
    println!("Waiting for heartbeat message...");
    if let Err(e) = timeout(Duration::from_secs(5), rx.recv()).await {
        println!("{}", Color::Red.paint(format!("Failed to start: {}", e)));
        exit(1);
    }

    // run chord
    Chord::run(Settings::default())
}

struct DiscordPipe;

impl prelude::TypeMapKey for DiscordPipe {
    type Value = mpsc::Sender<DiscordEvent>;
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
            .clone() // TODO: figure out why this is needed
            .send(DiscordEvent::MsgRecv(msg.content.clone()))
            .await
        {
            Ok(_) => println!("Sent message through channel: {}", msg.content),
            Err(e) => println!("{}", e),
        }
    }
}

// Application container
struct Chord {
    message_list: widgets::message_list::MessageList,
}

impl Application for Chord {
    type Executor = executor::Null;
    type Message = IntMessage;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Chord {
                message_list: widgets::message_list::MessageList::new(),
            },
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
                .push(Text::new(String::from("Receiving messages...")))
                .push(self.message_list.view()),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
    }
}
