use std::{collections::HashSet, time::SystemTime};

use chrono::{offset::Utc, DateTime};
use rand::{thread_rng, Rng};
use rql::prelude::*;
use serde_derive::{Deserialize, Serialize};
use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

macro_rules! command {
    ($($id:ident => $desc:literal),* $(,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        enum Command {
            $($id),*
        }
        impl Command {
            fn all_desc() -> String {
                let mut s = String::new();
                $(s.push_str(&format!("!{}: {}\n", stringify!($id).to_lowercase(), $desc));)*
                s
            }
        }
    }
}

command! {
    Help => "Display this message",
    Ping => "Ping the bot",
    Fortune => "Get your fortune told",
}

static FORTUNES: &[&str] = &[
    "###, you a good person deserving of love ❤",
    "###, every mistake you make is just an opportuniy to learn and better yourself 💪",
    "###, in times of trouble, turn to friends. \
     A good friend can make all the difference in your life 😌",
    "Fear not, ###, for tomorrow may always bring a better day. \
     If not tomorrow, then perhaps the day after that.",
    "###, honing your mind for problem solving will serve you well 🧠",
    "###, when something upsets you, ask yourself, \"Is this really \
     worth letting it affect me?\" More often that not, you may find \
     that the answer is no.",
    "###, a healthy interest in hobbies can be very fulfilling. \
     Even the most diehard weeb is lucky to have found something \
     to be passionate about, and can stem the onset of apathy.",
    "###, it can be helpful to write down your goals. Keeping track \
     of where you want to be in life can be an actuallizing process.",
    "###, it can be helpful to write down the things you don't want \
     to become. It is just as important to have something to run from as \
     it is to have something to run toward.",
    "###, it can be helpful, from time to time, to allow the \
     diffuculties of life to overcome you. Use this time for catharsis \
     and to build personal resolve.",
];

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
struct User {
    discord_id: u64,
    used_fortunes: HashSet<usize>,
    last_fortune_time: Option<DateTime<Utc>>,
}

schema! {
    Database {
        user: User
    }
}

struct Handler {
    db: Database,
}

impl Handler {
    fn user_id(&self, discord_id: u64) -> Id<User> {
        let id_op = self
            .db
            .user()
            .rows()
            .find(|user| user.discord_id == discord_id)
            .map(|user| user.id);
        if let Some(id) = id_op {
            id
        } else {
            self.db.user_mut().insert(User {
                discord_id,
                used_fortunes: HashSet::new(),
                last_fortune_time: None,
            })
        }
    }
}

impl Handler {
    fn handle_message(&self, ctx: Context, msg: Message) -> serenity::Result<()> {
        if msg.author.name == "Kai's Cool Bot" {
            return Ok(());
        }
        if msg.content.starts_with('!') {
            use Command::*;
            let command_str = &msg.content[1..];
            let mut words = command_str.split_whitespace();
            let command = if let Some(word) = words.next() {
                word
            } else {
                return Ok(());
            };
            match serde_yaml::from_str::<Command>(command) {
                Ok(command) => match command {
                    Help => {
                        msg.channel_id.say(&ctx.http, Command::all_desc())?;
                    }
                    Ping => {
                        msg.channel_id.say(&ctx.http, "Pong!")?;
                    }
                    Fortune => {
                        let user_id = self.user_id(*msg.author.id.as_u64());
                        let mut users = self.db.user_mut();
                        let user = users.get_mut(user_id).unwrap();
                        let tell_fortune = match user.last_fortune_time {
                            Some(ref mut last_time) => {
                                let time_since =
                                    DateTime::<Utc>::from(SystemTime::now()) - *last_time;
                                let hours = time_since.num_hours();
                                if hours >= 24 {
                                    *last_time = DateTime::from(SystemTime::now());
                                    Ok(())
                                } else {
                                    Err(24 - hours)
                                }
                            }
                            None => {
                                user.last_fortune_time = Some(DateTime::from(SystemTime::now()));
                                Ok(())
                            }
                        };
                        match tell_fortune {
                            Ok(()) => {
                                let name = match msg.author.name.as_str() {
                                    "Kai' Sa" => "Aether",
                                    "SkrubLyfe" => "Logan",
                                    "Most ok Kat NA" => "Trevor",
                                    "cokez11" => "Hamilton",
                                    "Kaikalii" => "Kai",
                                    s => s,
                                };
                                let mut rng = thread_rng();
                                if user.used_fortunes.len() == FORTUNES.len() {
                                    user.used_fortunes.clear();
                                }
                                let fortune = loop {
                                    let index = rng.gen_range(0, FORTUNES.len());
                                    if !user.used_fortunes.contains(&index) {
                                        user.used_fortunes.insert(index);
                                        break FORTUNES[index].replace("###", name);
                                    }
                                };
                                msg.channel_id.say(&ctx.http, fortune)?;
                            }
                            Err(hours) => {
                                msg.channel_id.say(
                                    &ctx.http,
                                    format!(
                                        "You have already had your fortune told today. \
                                         Try again in {} hours",
                                        hours
                                    ),
                                )?;
                            }
                        }
                    }
                },
                Err(_) => {
                    msg.channel_id
                        .say(&ctx.http, format!("Unknown command: {}", command_str))?;
                }
            }
        }
        Ok(())
    }
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        if let Err(e) = self.handle_message(ctx, msg) {
            println!("Error: {:?}", e);
        }
    }
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    let handler = Handler {
        db: Database::new("database", Representation::HumanReadable).unwrap(),
    };
    let mut client = Client::new(
        std::env::var("BOT_TOKEN").expect("You mut set the BOT_TOKEN environment variable"),
        handler,
    )
    .expect("Err creating client");

    if let Err(e) = client.start() {
        println!("Client error: {:?}", e);
    }
}
