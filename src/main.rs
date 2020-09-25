use ansi_term::Color;
use iced::{
    executor, Application, Column, Command, Container, Element, Length, Row, Settings, Text,
};
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{macros::group, StandardFramework},
    model::{channel::Message, gateway::Ready},
    prelude,
};
use std::{env, process::exit, thread, time::Duration};
use tokio::{runtime::Runtime, sync::mpsc, time::timeout};

use chord::common::{ChordWidget, DiscordEvent, Message as chordMessage};

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

pub enum Direction {
    Vertical,
    Horizontal,
}

// TODO: add a new()
pub struct Split {
    direction: Direction,
    ratio: u8,
}

// TODO: maybe this shouldn't be an enum? see line 224 for why
// TODO: determine a mechanism for widget->node interaction
pub enum Node {
    Leaf(Box<dyn ChordWidget>),
    Node(Split, Box<Node>, Box<Node>), // Left, Right or Top, Bottom
}

impl Node {
    // split node with el
    // TODO: choose where el goes, first or second
    fn split(&mut self, split: Split, el: Box<dyn ChordWidget>) {
        take_mut::take(self, |node| match node {
            Node::Leaf(sibling) => Node::Node(
                split,
                Box::new(Node::Leaf(sibling)),
                Box::new(Node::Leaf(el)),
            ),
            Node::Node(_, _, _) => Node::Node(split, Box::new(node), Box::new(Node::Leaf(el))),
        })
    }
}

impl ChordWidget for Node {
    // propogate messages
    fn update(&self, msg: chordMessage) {
        match self {
            Node::Leaf(widget) => {
                widget.update(msg);
            }
            Node::Node(_, widget_a, widget_b) => {
                widget_a.update(msg.clone());
                widget_b.update(msg);
            }
        }
    }

    // render node contents recursively
    fn view(&mut self) -> iced::Element<chordMessage> {
        match self {
            Node::Leaf(widget) => widget.view(),
            Node::Node(split, widget_a, widget_b) => match split.direction {
                Direction::Vertical => Row::new()
                    .push(
                        Container::new(widget_a.view())
                            .height(Length::Fill)
                            .width(Length::FillPortion(split.ratio.into()))
                            .center_x()
                            .center_y(),
                    )
                    .push(
                        Container::new(widget_b.view())
                            .height(Length::Fill)
                            .width(Length::FillPortion((100u8 - split.ratio).into()))
                            .center_x()
                            .center_y(),
                    )
                    .into(),
                Direction::Horizontal => Column::new()
                    .push(
                        Container::new(widget_a.view())
                            .width(Length::Fill)
                            .height(Length::FillPortion(split.ratio.into()))
                            .center_x()
                            .center_y(),
                    )
                    .push(
                        Container::new(widget_b.view())
                            .width(Length::Fill)
                            .height(Length::FillPortion((100u8 - split.ratio).into()))
                            .center_x()
                            .center_y(),
                    )
                    .into(),
            },
        }
    }
}

// Application container
struct Chord {
    root: Node,
}

impl Application for Chord {
    type Executor = executor::Null;
    type Message = chordMessage;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut chord = Chord {
            root: Node::Leaf(Box::new(widgets::message_list::MessageList::new())),
        };

        chord.root.split(
            Split {
                direction: Direction::Vertical,
                ratio: 50,
            },
            Box::new(widgets::message_list::MessageList::new()),
        );

        // :(
        if let Node::Node(_, ref mut one, ref _two) = chord.root {
            one.split(
                Split {
                    direction: Direction::Horizontal,
                    ratio: 80,
                },
                Box::new(widgets::message_list::MessageList::new()),
            )
        }

        return (chord, Command::none());
    }

    fn title(&self) -> String {
        String::from("Chord")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.root.view().explain(iced::Color::BLACK)
    }
}
