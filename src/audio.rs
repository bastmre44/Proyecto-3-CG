//! rodio 0.21.1: https://docs.rs/rodio/0.21.1/rodio/
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};
enum Command {
    Volume(f32),
}
#[derive(Clone, Copy)]
struct Volume {
    level: f32,
    muted: bool,
}
impl Volume {
    fn effective(self) -> f32 {
        if self.muted { 0.0 } else { self.level }
    }
    fn change(&mut self, delta: f32) {
        self.level = (self.level + delta).clamp(0.0, 1.0);
    }
}
pub struct Audio {
    commands: Sender<Command>,
    reports: Receiver<Result<(), String>>,
    volume: Volume,
    pub status: String,
}
impl Audio {
    pub fn start(path: PathBuf) -> Self {
        let (commands, rx) = mpsc::channel();
        let (tx, reports) = mpsc::channel();
        thread::spawn(move || {
            let result = (|| {
                let file =
                    std::fs::File::open(&path).map_err(|e| format!("MP3 no disponible: {e}"))?;
                let decoder = Decoder::try_from(file).map_err(|e| format!("MP3 invalido: {e}"))?;
                let stream = OutputStreamBuilder::open_default_stream()
                    .map_err(|e| format!("Sin dispositivo de audio: {e}"))?;
                let sink = Sink::connect_new(stream.mixer());
                sink.set_volume(0.25);
                // Playback and decoding happen on rodio's audio thread. This source is appended only once.
                sink.append(decoder.repeat_infinite());
                let _ = tx.send(Ok(()));
                while let Ok(Command::Volume(volume)) = rx.recv() {
                    sink.set_volume(volume);
                }
                drop(sink);
                drop(stream);
                Ok::<(), String>(())
            })();
            if let Err(error) = result {
                eprintln!("Audio: {error}");
                let _ = tx.send(Err(error));
            }
        });
        Self {
            commands,
            reports,
            volume: Volume {
                level: 0.25,
                muted: false,
            },
            status: "AUDIO: INICIANDO".into(),
        }
    }
    pub fn update(&mut self, mute: bool, delta: f32) {
        if mute {
            self.volume.muted = !self.volume.muted;
        }
        self.volume.change(delta);
        if mute || delta != 0.0 {
            let _ = self.commands.send(Command::Volume(self.volume.effective()));
        }
        while let Ok(report) = self.reports.try_recv() {
            self.status = match report {
                Ok(()) => "AUDIO: LISTO".into(),
                Err(_) => "AUDIO NO DISPONIBLE - VER CONSOLA".into(),
            };
        }
    }
    pub fn label(&self) -> String {
        format!(
            "{} | {} {}% | M: SILENCIO  -/+: VOLUMEN",
            self.status,
            if self.volume.muted { "MUDO" } else { "VOL" },
            (self.volume.level * 100.0).round() as u32
        )
    }
}
pub fn music_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/audio/musica.mp3")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn silencio_conserva_volumen_y_limites() {
        let mut v = Volume {
            level: 0.25,
            muted: false,
        };
        assert_eq!(v.effective(), 0.25);
        v.muted = true;
        assert_eq!(v.effective(), 0.0);
        v.change(0.1);
        v.muted = false;
        assert!((v.effective() - 0.35).abs() < 0.00001);
        v.change(10.0);
        assert_eq!(v.level, 1.0);
        v.change(-10.0);
        assert_eq!(v.level, 0.0);
    }
    #[test]
    fn archivo_ausente_no_impide_continuar() {
        let mut audio = Audio::start(std::env::temp_dir().join("showgirl-no-existe-839102.mp3"));
        let report = audio
            .reports
            .recv_timeout(std::time::Duration::from_secs(3))
            .unwrap();
        assert!(report.unwrap_err().contains("MP3 no disponible"));
        audio.update(true, 0.1);
    }
    #[test]
    fn mp3_real_se_decodifica() {
        let file = std::fs::File::open(music_path()).unwrap();
        let mut source = Decoder::try_from(file).unwrap();
        assert!(source.next().is_some());
        assert!(source.channels() > 0);
    }
}
