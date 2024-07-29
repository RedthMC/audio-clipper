mod error;
mod voice;

use std::{env, sync::Arc};

use anyhow::Result;
use dotenv::dotenv;
use serenity::{
    all::{CreateAttachment, CreateMessage},
    async_trait,
    client::{Client, Context, EventHandler},
    model::channel::Message,
    prelude::GatewayIntents,
};

use songbird::{driver::DecodeMode, Config, SerenityInit};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN")?;
    let intents = GatewayIntents::all();
    let songbird_config = Config::default().decode_mode(DecodeMode::Decode);

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler::default())
        .register_songbird_from_config(songbird_config)
        .await?;

    client.start().await?;

    Ok(())
}

#[derive(Default)]
struct Handler {
    voice_clipper: Arc<Mutex<voice::VoiceClipper>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        let guild_id = message.guild_id.expect("no guild");
        let mut vc = self.voice_clipper.lock().await;
        match message.content.as_str() {
            "!join" => vc.join(&ctx, guild_id, message.channel_id).await.unwrap(),
            "!leave" => vc.leave(&ctx, guild_id, message.channel_id).await.unwrap(),
            "!clip" => match vc.clip(message.channel_id) {
                Ok(bytes) => {
                    let file = CreateAttachment::bytes(bytes, "output.mp3");
                    let reply = CreateMessage::new().add_file(file).reference_message(&message);
    
                    message.channel_id.send_message(ctx, reply).await.expect("reply failed");
                }
                Err(error::Error::NoConnection) => {
                    message.reply(ctx, "!join first").await.expect("reply failed");
                }
                Err(_) => {
                    message.reply(ctx, "internal issue").await.expect("reply failed");
                }
            },
            _ => {}
        };
    }
}
