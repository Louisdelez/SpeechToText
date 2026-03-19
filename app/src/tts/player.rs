use std::io::BufReader;

use rodio::{Decoder, OutputStream, Sink};

pub fn play_wav(path: &str) -> Result<(), String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Ouverture audio: {e}"))?;
    let reader = BufReader::new(file);

    let (_stream, handle) =
        OutputStream::try_default().map_err(|e| format!("Sortie audio: {e}"))?;
    let sink = Sink::try_new(&handle).map_err(|e| format!("Sink audio: {e}"))?;
    let source = Decoder::new(reader).map_err(|e| format!("Decodage audio: {e}"))?;

    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

pub fn play_wav_async(path: String) {
    std::thread::spawn(move || {
        if let Err(e) = play_wav(&path) {
            eprintln!("[TTS] Erreur lecture audio: {e}");
        }
        // Clean up the temp file after playback
        let _ = std::fs::remove_file(&path);
    });
}
