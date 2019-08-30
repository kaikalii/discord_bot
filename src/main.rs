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
    Advice => "Get your fortune told",
    Fortune => "Replaced with !advice",
}

static ADVICE: &[&str] = &[
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
    "###, home is not a place. Home is the people with which you spend \
     you life, the people that you support and those that support you. \
     You can live almost anywhere as long as you have family or friends.",
    "###, even if you can get something done with a bad attitude, \
     wouldn't it be nicer to find a positive outlook on it?",
    "###, a 1v5 is not winnable, but a 2v5 is. Anything is possible \
     when you have a friend by your side.",
    "###, one day the Dark Lord will rise again, and he will rain fire \
     on the mountains of man and sit on the throne of the Earth.",
    "###, before making a big decision, you may want to wait an amount \
     of time proportional to how long that decision will affect your life. \
     You may find that it is not what you wanted after all.",
    "###, there are people in the world who have perfect memories and are \
     unable to forget anything. Make no mistake, this is a terrible curse. \
     The ability to forget is the ability to move on.",
    "###, Don't keep a bad friend around out of nothing but loyalty. \
     Someone who consistently makes the same bad decisions that burden \
     you is not worthy of your friendship, and can be cut off without remorse.",
    "###, very few things a actually impossible. A great many things are \
     very improbable. The challenge of the realistic optimist is deciding how \
     unlikely is too unlikely.",
    "###, a game that seems lost at 20 minutes could be come back from and \
     won a 50 minutes. Is it worth it though? You could just surrender at 20 \
     and use that 30 minutes to play a new game.",
    "###, whenever you are dismayed at your own lack of initiative, remember \
     the words of a wise old sage: \"Yesterday you said tomorrow, so JUST DO IT!\"",
    "###, when you find someone's behavior appalling, it may be useful to use \
     words to get to the root of why they would do such a thing. Some people are \
     beyond saving, others are actually reasonable and can be reformed via \
     conversation.",
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
                    Advice => {
                        let user_id = self.user_id(*msg.author.id.as_u64());
                        let mut users = self.db.user_mut();
                        let user = users.get_mut(user_id).unwrap();
                        let tell_advice = match user.last_fortune_time {
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
                        match tell_advice {
                            Ok(()) => {
                                let name = match msg.author.name.as_str() {
                                    "Kai' Sa" => "Aether",
                                    "Skrublyfe" => "Logan",
                                    "Most ok Kat NA" => "Trevor",
                                    "cokez11" => "Hamilton",
                                    "Kaikalii" => "Kai",
                                    "[Mr.Dr.SistrFistr]" => "Kiernan",
                                    "Boosted Bonobo" => "Jimmy",
                                    s => s,
                                };
                                let mut rng = thread_rng();
                                if user.used_fortunes.len() == ADVICE.len() {
                                    user.used_fortunes.clear();
                                }
                                let mut user_table = self.db.user_mut();
                                let mut meta_user = user_table
                                    .update()
                                    .find(|user| user.discord_id == 0)
                                    .expect("No meta user");
                                let advice = loop {
                                    let index = rng.gen_range(0, ADVICE.len());
                                    if !user.used_fortunes.contains(&index)
                                        && !meta_user.used_fortunes.contains(&index)
                                    {
                                        user.used_fortunes.insert(index);
                                        meta_user.used_fortunes.insert(index);
                                        break ADVICE[index].replace("###", name);
                                    }
                                };
                                msg.channel_id.say(&ctx.http, advice)?;
                            }
                            Err(hours) => {
                                msg.channel_id.say(
                                    &ctx.http,
                                    format!(
                                        "You have already been given advice today. \
                                         Try again in {} hours",
                                        hours
                                    ),
                                )?;
                            }
                        }
                    }
                    Fortune => {
                        msg.channel_id.say(
                            &ctx.http,
                            "The !fortune command has been replaced with !advice.",
                        )?;
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
        if !self.db.user().find(|user| user.discord_id == 0).is_some() {
            let used_fortunes: HashSet<usize> = self
                .db
                .user()
                .rows()
                .flat_map(|user| user.used_fortunes.clone())
                .collect();
            self.db.user_mut().insert(User {
                discord_id: 0,
                used_fortunes,
                last_fortune_time: None,
            });
        }
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
