use narwhalol::{constants::Region, dto::api::Summoner, synchronous::client::LeagueAPI};
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
struct User {
    summoner_name: Option<String>,
}

schema! {
    Database {
        user: User
    }
}

struct Handler {
    league: LeagueAPI,
    db: Database,
}

impl Handler {
    fn handle_message(&self, ctx: Context, msg: Message) -> serenity::Result<()> {
        if msg.author.name == "Kai's Cool Bot" {
            return Ok(());
        }
        if msg.content.starts_with('!') {
            use Command::*;
            let command_str = &msg.content[1..];
            match serde_yaml::from_str::<Command>(command_str) {
                Ok(command) => match command {
                    Help => {
                        msg.channel_id.say(&ctx.http, Command::all_desc())?;
                    }
                    Ping => {
                        msg.channel_id.say(&ctx.http, "Pong!")?;
                    }
                    Fortune => {
                        let name = match msg.author.name.as_str() {
                            "Kai' Sa" => "Aether",
                            "SkrubLyfe" => "Logan",
                            "Most ok Kat NA" => "Trevor",
                            "cokez11" => "Hamilton",
                            "Kaikalii" => {
                                msg.channel_id.say(
                                    &ctx.http,
                                    "Kai is a bastion of masculinity and social poise",
                                )?;
                                return Ok(());
                            }
                            s => s,
                        };
                        msg.channel_id.say(&ctx.http, format!("{} is gay", name))?;
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
    std::env::set_var("RIOT_API_KEY", "RGAPI-af8b654e-8929-4e17-9c8f-ef682fe0bac4");
    let handler = Handler {
        league: LeagueAPI::new(Region::NA),
        db: Database::new("database", Representation::HumanReadable).unwrap(),
    };
    let mut client = Client::new(
        "NjA5NjE0MTg0Mzg4MzYyMjQy.XU5ddQ.Ia3BAYgGu_tVI5tKEERdEbyqdDE",
        handler,
    )
    .expect("Err creating client");

    if let Err(e) = client.start() {
        println!("Client error: {:?}", e);
    }
}
