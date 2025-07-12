mod commands;

use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};

use crate::commands::stats;

struct Bot {
    api_key: String,
    client: reqwest::Client,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("ONLINE: ") {
            // Analyse the /who command for bedwars stats
            let csv_igns = &msg.content[8..];
            let igns = csv_igns.split(", ");
            let mut message = String::new();
            for ign in igns {
                match stats::get_stats(ign, &self.client, &self.api_key).await {
                    Ok(msg) => message.push_str(format!("{}\n", msg).as_str()),
                    Err(e) => message.push_str(format!("Error: {}\n", e).as_str())
                };
            }
            if let Err(e) = msg.channel_id.say(&ctx.http, message).await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
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
    let api_key = secrets
        .get("HYPIXEL_API_KEY")
        .context("'HYPIXEL_API_KEY' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot {
            client: reqwest::Client::new(),
            api_key
        })
        .await
        .expect("Err creating client");

    Ok(client.into())
}
