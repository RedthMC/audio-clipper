use std::{
    mem::MaybeUninit,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use mp3lame_encoder as mp3;
use serenity::{
    all::{ChannelId, Context, GuildId},
    async_trait,
};
use songbird::{events::context_data::VoiceTick, CoreEvent, Event, EventContext, EventHandler};
use tokio::time::sleep;

pub async fn join(ctx: &Context, guild_id: GuildId, channel_id: ChannelId) -> Box<[u8]> {
    let arc = Arc::new(Mutex::new(VoiceRecorder::default()));

    let sb = songbird::get(ctx).await.expect("no songbird");
    sb.join(guild_id, channel_id)
        .await
        .expect("failed connection")
        .lock()
        .await
        .add_global_event(
            CoreEvent::VoiceTick.into(),
            Handler {
                voice_recorder: arc.clone(),
            },
        );
    sleep(Duration::from_secs(10)).await;
    println!("disconnecting!!!!");
    sb.remove(guild_id).await.expect("failed to leave");

    let buf = arc
        .lock()
        .expect("cannot obtain recorder")
        .writer
        .take()
        .expect("no writer");
    let input = mp3::InterleavedPcm(&buf);

    let mut encoder = mp3::Builder::new().expect("Create LAME builder");
    encoder.set_sample_rate(48000).expect("set sample rate");
    encoder.set_brate(mp3::Bitrate::Kbps192).expect("set brate");
    encoder
        .set_quality(mp3::Quality::Best)
        .expect("set quality");
    let mut mp3_encoder = encoder.build().expect("To initialize LAME encoder");

    let cap = mp3::max_required_buffer_size(buf.len());
    let mut output = vec![MaybeUninit::uninit(); cap];
    let body_len = mp3_encoder
        .encode(input, &mut output)
        .expect("encoding failed");
    let header_len = mp3_encoder
        .flush::<mp3::FlushNoGap>(&mut output)
        .expect("flush failed");
    println!("finalized!!!!");
    output.truncate(header_len + body_len);
    return output
        .iter()
        .map(|b| unsafe { b.assume_init_read() })
        .collect();
}

#[derive(Default)]
struct Handler {
    voice_recorder: Arc<Mutex<VoiceRecorder>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let tick = match ctx {
            EventContext::VoiceTick(vt) => vt,
            _ => return None,
        };
        self.voice_recorder
            .lock()
            .expect("voice recorder not obtained")
            .write(tick);
        None
    }
}

struct VoiceRecorder {
    last_timestamp: Instant,
    writer: Option<Vec<i16>>,
}

impl VoiceRecorder {
    fn write(&mut self, voice_tick: &VoiceTick) -> Option<()> {
        let writer = self.writer.as_mut()?;
        if voice_tick.speaking.len() == 0 {
            return None;
        }
        let mut samples = vec![0i16; 1920];

        for data in voice_tick.speaking.values() {
            for (index, &sample) in data.decoded_voice.as_ref()?.iter().enumerate() {
                samples[index] = samples[index].saturating_add(sample);
            }
        }

        Self::pad_silence(writer, self.last_timestamp);
        self.last_timestamp = Instant::now();
        writer.extend_from_slice(&samples);

        None
    }

    fn pad_silence(writer: &mut Vec<i16>, last_time: Instant) -> Option<()> {
        let elapsed_ms = last_time
            .elapsed()
            .saturating_sub(Duration::from_millis(20))
            .as_millis() as usize;
        if elapsed_ms < 20 { return None; }
        writer.extend(vec![0i16; elapsed_ms * 96]);
        None
    }
}
impl Default for VoiceRecorder {
    fn default() -> Self {
        Self {
            last_timestamp: Instant::now(),
            writer: Some(Vec::new()),
        }
    }
}
