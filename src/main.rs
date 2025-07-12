mod commands;

use std::collections::HashMap;

use anyhow::Context as _;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};

use crate::commands::stats;

struct Bot {
    api_key: String,
    client: reqwest::Client,
    guild_id: String,
}

struct IgnoreList;

impl TypeMapKey for IgnoreList {
    type Value = HashMap<String, Vec<String>>;
}

#[async_trait]
impl EventHandler for Bot {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "ignore" => Some(commands::ignore::run(&command.data.options(), &ctx).await),
                "ignorelist" => Some(commands::ignorelist::run(&ctx).await),
                _ => Some("Unknown command".into())
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(err) = command.create_response(&ctx.http, builder).await {
                    println!("Error responding to command: {:?}", err)
                }
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("ONLINE: ") {
            // Analyse the /who command for bedwars stats
            let csv_igns = &msg.content[8..];
            let igns = csv_igns.split(", ");
            let mut message = String::from("```ansi\n");
            let data = ctx.data.read().await;
            let ignore_list_map = data.get::<IgnoreList>().unwrap();
            let ignore_list = ignore_list_map.get("IgnoreList").unwrap();
            for ign in igns {
                if !ignore_list.contains(&ign.to_string()) {
                    match stats::get_stats(ign, &self.client, &self.api_key).await {
                        Ok(msg) => message.push_str(format!("{}\n", msg).as_str()),
                        Err(e) => message.push_str(format!("Error: {}\n", e).as_str())
                    };
                }
            }
            message.push_str("```");
            if let Err(e) = msg.channel_id.say(&ctx.http, message).await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(self.guild_id.parse().expect("Guild ID must be an integer"));

        let commands = guild_id.set_commands(
            &ctx.http,
            vec![
                commands::ignore::register(),
                commands::ignorelist::register(),
            ]
        ).await;

        println!("Setup commands: {:?}", commands);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;
    let guild_id = secrets
        .get("DISCORD_GUILD_ID")
        .context("'DISCORD_GUILD_ID' was not found")?;
    let api_key = secrets
        .get("HYPIXEL_API_KEY")
        .context("'HYPIXEL_API_KEY' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot {
            client: reqwest::Client::new(),
            api_key,
            guild_id,
        })
        .await
        .expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<IgnoreList>(HashMap::default());
    }

    Ok(client.into())
}
