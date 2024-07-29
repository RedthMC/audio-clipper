use std::{
    collections::{HashMap, VecDeque},
    mem::MaybeUninit,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::error::{Error, Result};
use mp3lame_encoder::{self as mp3, InterleavedPcm};
use serenity::{
    all::{ChannelId, Context as Discord, GuildId},
    async_trait,
};
use songbird::{events::context_data::VoiceTick, CoreEvent, Event, EventContext, EventHandler};

#[derive(Default)]
pub struct VoiceClipper {
    map: HashMap<ChannelId, Arc<Mutex<VoiceRecorder>>>,
}

impl VoiceClipper {
    pub async fn join(&mut self, ctx: &Discord, guild_id: GuildId, channel_id: ChannelId) -> Result<()> {
        let arc = Arc::new(Mutex::new(VoiceRecorder::default()));

        let sb = songbird::get(ctx).await.ok_or(Error::CantConnect)?;
        sb.join(guild_id, channel_id).await?.lock().await.add_global_event(
            CoreEvent::VoiceTick.into(),
            Handler {
                voice_recorder: arc.clone(),
            },
        );

        self.map.insert(channel_id, arc);

        Ok(())
    }

    pub async fn leave(&mut self, ctx: &Discord, guild_id: GuildId, channel_id: ChannelId) -> Result<()> {
        let sb = songbird::get(ctx).await.ok_or(Error::CantConnect)?;
        match self.map.remove(&channel_id) {
            Some(_) => sb.leave(guild_id).await?,
            None => {},
        };
        Ok(())
    }

    fn get_audio(&self, channel_id: ChannelId) -> Result<VecDeque<i16>> {
        Ok(self.map.get(&channel_id).ok_or(Error::NoConnection)?.lock()?.writer.clone())
    }

    pub fn clip(&self, channel_id: ChannelId) -> Result<Box<[u8]>> {
        let vec_deque = &mut self.get_audio(channel_id)?;
        let audio = vec_deque.make_contiguous();
        let input = InterleavedPcm(audio);

        let mut encoder = mp3::Builder::new().ok_or(mp3::BuildError::NoMem)?;
        encoder.set_sample_rate(48000)?;
        encoder.set_brate(mp3::Bitrate::Kbps192)?;
        encoder.set_quality(mp3::Quality::Best)?;
        let mut mp3_encoder = encoder.build()?;

        let cap = mp3::max_required_buffer_size(audio.len());
        let mut output = vec![MaybeUninit::uninit(); cap];
        let body_len = mp3_encoder.encode(input, &mut output)?;
        let header_len = mp3_encoder.flush::<mp3::FlushNoGap>(&mut output)?;
        println!("finalized!!!!");

        output.truncate(header_len + body_len);
        return Ok(output.iter().map(|b| unsafe { b.assume_init_read() }).collect());
    }
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
        self.voice_recorder.lock().expect("voice recorder not obtained").write(tick);
        None
    }
}

struct VoiceRecorder {
    last_timestamp: Instant,
    writer: VecDeque<i16>,
}

impl VoiceRecorder {
    fn write(&mut self, voice_tick: &VoiceTick) -> Option<()> {
        if voice_tick.speaking.len() == 0 {
            return None;
        }
        let mut samples = vec![0i16; 1920];

        for data in voice_tick.speaking.values() {
            for (index, &sample) in data.decoded_voice.as_ref()?.iter().enumerate() {
                samples[index] = samples[index].saturating_add(sample);
            }
        }

        Self::pad_silence(&mut self.writer, self.last_timestamp);
        self.last_timestamp = Instant::now();
        self.writer.extend(samples);

        while self.writer.len() > 10 * 1000 * 96 {
            self.writer.pop_front();
        }
        None
    }

    fn pad_silence(writer: &mut VecDeque<i16>, last_time: Instant) -> Option<()> {
        let elapsed_ms = last_time.elapsed().saturating_sub(Duration::from_millis(20)).as_millis() as usize;
        if elapsed_ms < 20 {
            return None;
        }
        writer.extend(vec![0i16; elapsed_ms * 96]);
        None
    }
}
impl Default for VoiceRecorder {
    fn default() -> Self {
        Self {
            last_timestamp: Instant::now(),
            writer: VecDeque::new(),
        }
    }
}
