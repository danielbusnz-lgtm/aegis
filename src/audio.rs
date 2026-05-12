use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Play raw audio bytes (WAV or MP3) through the default output sink.
/// Writes to a temp file first because rodio's symphonia decoder is
/// picky about in-memory Cursor<Vec<u8>> input.
///
/// Interruptible: if `hotkey::is_recording()` becomes true while audio
/// is playing, playback stops immediately and the function returns Ok.
/// This lets the user cut off the assistant mid-sentence by pressing
/// the talk hotkey.
pub fn play(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let path = "/tmp/aegis-playback.wav";
    std::fs::write(path, bytes)?;
    let file = std::io::BufReader::new(std::fs::File::open(path)?);
    let handle = rodio::DeviceSinkBuilder::open_default_sink()?;
    let player = rodio::play(handle.mixer(), file)?;

    while !player.empty() {
        if crate::hotkey::is_recording() {
            player.stop();
            eprintln!("[audio] playback interrupted by hotkey");
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

pub fn record_until_release() -> (Vec<f32>, u32, u16) {
    let t0 = std::time::Instant::now();
    let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let buffer_for_callback = Arc::clone(&buffer);

    let host = cpal::default_host();
    let device = host
        .input_devices()
        .expect("could not list input devices")
        .find(|d| {
            d.name()
                .map(|n| n == "pulse" || n == "pipewire")
                .unwrap_or(false)
        })
        .or_else(|| host.default_input_device())
        .expect("no input device available");
    eprintln!("[audio] picked device in {:?}", t0.elapsed());

    let t1 = std::time::Instant::now();
    let supported_config = device
        .default_input_config()
        .expect("no default input config");
    eprintln!("[audio] got config in {:?}", t1.elapsed());
    let sample_rate = supported_config.sample_rate();
    let channels = supported_config.channels();
    let config = supported_config.config();

    let t2 = std::time::Instant::now();
    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buf = buffer_for_callback.lock().unwrap();
                buf.extend_from_slice(data);
            },
            move |err| eprintln!("audio stream error: {}", err),
            None,
        )
        .expect("failed to build input stream");
    eprintln!("[audio] built stream in {:?}", t2.elapsed());

    let t3 = std::time::Instant::now();
    stream.play().expect("failed to start stream");
    eprintln!("[audio] stream playing in {:?} (total setup: {:?})", t3.elapsed(), t0.elapsed());
    println!("recording... (release to stop)");

    while crate::hotkey::is_recording() {
        thread::sleep(Duration::from_millis(10));
    }
    println!("recording stopped");

    let samples = buffer.lock().unwrap().clone();
    (samples, sample_rate, channels)
}
