mod model;
mod persistence;

use model::{
    accounts::{Accounts, AccountsType},
    commands::{
        balance::{balance_command, balance_handler},
        bill::{bill_command, bill_handler},
        owe::{owe_command, owe_handler},
        HandleCommandError,
    },
};

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    futures::StreamExt,
    model::{
        gateway::Ready,
        interactions::{
            application_command::ApplicationCommand, Interaction,
            InteractionResponseType::ChannelMessageWithSource,
        },
    },
};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;

use dotenv::dotenv;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "balance" => balance_handler(&ctx, &command).await,
                "owe" => owe_handler(&ctx, &command).await,
                "bill" => bill_handler(&ctx, &command).await,
                _ => Err(HandleCommandError),
            };

            let content = match content {
                Ok(content) => content,
                Err(_e) => "Error handling command".to_string(),
            };

            if let Err(e) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        {
            let mut data = ctx.data.write().await;
            data.insert::<Accounts>(AccountsType::new());
        }

        let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(balance_command)
                .create_application_command(owe_command)
                .create_application_command(bill_command)
        })
        .await;

        println!("Loaded {} commands.", commands.unwrap_or(Vec::new()).len());
    }
}

async fn handle_signals(signals: Signals) {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT => {
                std::process::exit(0);
            }
            _ => unreachable!(),
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let signals = match Signals::new(&[SIGTERM, SIGINT]) {
        Ok(signals) => signals,
        Err(_e) => std::process::exit(1),
    };
    tokio::spawn(handle_signals(signals));

    let token = std::env::var("DISCORD_TOKEN").expect("token");

    let application_id: u64 = std::env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    if let Err(e) = client.start().await {
        println!("An error occurred while running the client: {:?}", e);
    }
}
