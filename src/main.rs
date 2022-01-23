use std::env;
use std::ptr::null;

use chrono::{Date, DateTime, Local, TimeZone, Utc};
use select::node::Data;
use serenity::{async_trait, FutureExt, model::{
    gateway::Ready,
    id::GuildId,
    interactions::{
        application_command::{
            ApplicationCommand,
            ApplicationCommandInteractionDataOptionValue,
            ApplicationCommandOptionType,
        },
        Interaction,
        InteractionResponseType,
    },
}, prelude::*};

mod vlscrapper;

struct Tekma {
    date: Date<Local>,
    location: String,
    organizer: String,
    league: String,
}

struct AgamaEvtHandler;

#[async_trait]
impl EventHandler for AgamaEvtHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "ping" => "Hey, I'm alive!".to_string(),
                "id" => {
                    let options = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected user option")
                        .resolved
                        .as_ref()
                        .expect("Expected user object");

                    if let ApplicationCommandInteractionDataOptionValue::User(user, _member) =
                    options
                    {
                        format!("{}'s id is {}", user.tag(), user.id)
                    } else {
                        "Please provide a valid user".to_string()
                    }
                },
                "tekma" => {
                    let options = command
                        .data
                        .options
                        .get(0)
                        .expect("Hmm, pričakovala sem ligo TEKME.")
                        .resolved
                        .as_ref()
                        .expect("Liga TEKME pričakovana ...");

                    if let ApplicationCommandInteractionDataOptionValue::String(liga) =
                    options {
                        match liga.as_str() {
                            // Vzhodna liga
                            "vl" => unsafe {
                                let options = command
                                    .data
                                    .options
                                    .get(1);

                                let number: i32= match options {
                                    Some(option) => {
                                        if let ApplicationCommandInteractionDataOptionValue::Integer(podana) =
                                        option.resolved.as_ref().unwrap() {
                                            *podana as i32
                                        } else {
                                            -1 as i32
                                        }
                                    },
                                    None => -1 as i32
                                };

                                // Dejanski index tekme (programerski)
                                let dejanska_tekma: usize = (number - 1) as usize;
                                if dejanska_tekma < TEKME_VL.len() {
                                    // Pridobi tekmo na dejanskem indexu
                                    TEKME_VL.get(dejanska_tekma).map(|tekma| {
                                        format!(
                                            "Tekma {}.: {} ({})",
                                            number,
                                            tekma.location,
                                            tekma.date.format("%d. %m. %Y")
                                        )
                                    }).unwrap_or_else(|| "Tekma ne obstaja".to_string())
                                } else if number == 0 {
                                    // Print naslednje tekme
                                    // Pridobi današnji datum
                                    let now: Date<Local> = Local::today();
                                    let mut naslednja = None;

                                    for tekma in &TEKME_VL {
                                        if tekma.date > now {
                                            naslednja = Some(tekma);
                                            "Naslednja tekma: ".to_string();
                                            break;
                                        }
                                    }

                                    if naslednja.is_some() {
                                        let tekma = naslednja.unwrap();
                                        format!(
                                            "Naslednja tekma: {} ({})",
                                            tekma.location,
                                            tekma.date.format("%d. %m. %Y")
                                        )
                                    } else {
                                        "Tekem je za to sezono konec.".to_string()
                                    }
                                } else if number < 0 {
                                    // Vse tekme
                                    let mut tekme = String::new();
                                    for (index, tekma) in TEKME_VL.iter().enumerate() {
                                        tekme.push_str(&format!(
                                            "{}. {} ({})\n",
                                            index + 1,
                                            tekma.location,
                                            tekma.date.format("%d. %m. %Y")
                                        ));
                                    }
                                    tekme
                                } else {
                                    "Tekma ne obstaja.".to_string()
                                }
                            },

                            // Drzavno prvenstvo
                            "dp" => {
                                "Tekma ne obstaja".to_string()
                            },
                            _ => {
                                "Liga ne obstaja".to_string()
                            }
                        }
                    } else {
                        "Please provide a valid league".to_string()
                    }
                },
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let _commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("tekma").description("Pregled tekem plezanja")
                        .create_option(|option| {
                            option
                                .name("tip")
                                .description("Liga tekme.")
                                .kind(ApplicationCommandOptionType::String)
                                .add_string_choice("vzhodna liga", "vl")
                                .add_string_choice("državno prvenstvo", "dp")
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("stevilka")
                                .description("Številka tekme..")
                                .kind(ApplicationCommandOptionType::Integer)
                                .min_int_value(0)
                        })
                })
                .create_application_command(|command| {
                    command.name("id").description("Get a user id").create_option(|option| {
                        option
                            .name("id")
                            .description("The user to lookup")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
                })
                .create_application_command(|command| {
                    command
                        .name("welcome")
                        .description("Welcome a user")
                        .create_option(|option| {
                            option
                                .name("user")
                                .description("The user to welcome")
                                .kind(ApplicationCommandOptionType::User)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("message")
                                .description("The message to send")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                                .add_string_choice(
                                    "Welcome to our cool server! Ask me if you need help",
                                    "pizza",
                                )
                                .add_string_choice("Hey, do you want a coffee?", "coffee")
                                .add_string_choice(
                                    "Welcome to the club, you're now a good person. Well, I hope.",
                                    "club",
                                )
                                .add_string_choice(
                                    "I hope that you brought a controller to play together!",
                                    "game",
                                )
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("numberinput")
                        .description("Test command for number input")
                        .create_option(|option| {
                            option
                                .name("int")
                                .description("An integer from 5 to 10")
                                .kind(ApplicationCommandOptionType::Integer)
                                .min_int_value(5)
                                .max_int_value(10)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("number")
                                .description("A float from -3.3 to 234.5")
                                .kind(ApplicationCommandOptionType::Number)
                                .min_number_value(-3.3)
                                .max_number_value(234.5)
                                .required(true)
                        })
                })
        }).await;


        let _global = ApplicationCommand::create_global_application_command(&ctx.http, |command| {
            command.name("wonderful_command").description("An amazing command")
        }).await;

    }
}

static mut TEKME_VL: Vec<Tekma> = Vec::new();

#[tokio::main]
async fn main() {
    unsafe {
        TEKME_VL = vlscrapper::pridobi_vl_tekme().await;
    }

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    // Build our client.
    let mut client = Client::builder(token)
        .event_handler(AgamaEvtHandler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

}
