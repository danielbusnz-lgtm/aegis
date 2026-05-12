use crate::audio;
use crate::cursor;
use crate::hotkey;
use crate::providers::claude::Claude;
use crate::providers::tts_cartesia::TtsCartesia;
use crate::providers::whisper_openai::WhisperOpenAi;
use crate::providers::{Stt, Tts};
use crate::screenshot;

pub fn run_loop(whisper: WhisperOpenAi, claude: Claude, cartesia: TtsCartesia) {
    println!("aegis ready — hold SUPER+space to talk");
    loop {
        hotkey::wait_for_press();
        if let Err(e) = run_one_turn(&whisper, &claude, &cartesia) {
            eprintln!("voice turn failed: {}", e);
        }
    }
}

fn run_one_turn(
    whisper: &WhisperOpenAi,
    claude: &Claude,
    cartesia: &TtsCartesia,
) -> Result<(), Box<dyn std::error::Error>> {
    let (samples, sr, ch) = audio::record_until_release();
    let transcript = whisper.transcribe(&samples, sr, ch)?;
    println!("you said: {}", transcript);

    let (b64, w, h) = screenshot::capture_active_workspace()?;
    let (text, point) = claude.ask_with_image_tool(&transcript, &b64)?;
    println!("claude: {}", text);

    if let Some((x, y)) = point {
        let cx = x.clamp(0, w as i32 - 1);
        let cy = y.clamp(0, h as i32 - 1);
        cursor::point_at(cx, cy);
    }

    if !text.is_empty() {
        let wav = cartesia.synthesize(&text)?;
        audio::play(&wav)?;
    }

    Ok(())
}
