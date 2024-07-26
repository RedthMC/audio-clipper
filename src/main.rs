mod voice;

use std::env;

use dotenv::dotenv;
use serenity::{
    all::{CreateAttachment, CreateMessage},
    async_trait,
    client::{Client, Context, EventHandler},
    model::{channel::Message, gateway::Ready},
    prelude::GatewayIntents,
};

use songbird::{driver::DecodeMode, Config, SerenityInit};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::all();
    let songbird_config = Config::default().decode_mode(DecodeMode::Decode);

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .register_songbird_from_config(songbird_config)
        .await
        .expect("Err creating client");

    client.start().await.expect("Client ended: ");
}

struct Handler;wwwwww

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, message: Message) {
        if message.content != "!clip" {
            return;
        }

        let bytes = voice::join(
            &ctx,
            message.guild_id.expect("no guild"),
            message.channel_id,
        )
        .await;

        let file = CreateAttachment::bytes(bytes, "output.mp3");
        let reply = CreateMessage::new()
            .add_file(file)
            .reference_message(&message);

        message
            .channel_id
            .send_message(ctx, reply)
            .await
            .expect("reply failed");
    }
}
