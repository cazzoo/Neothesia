use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("MIDI file error: {0}")]
    MidiError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid MIDI file: {0}")]
    InvalidMidi(String),
}

pub trait MidiParser: Send + Sync {
    fn parse_metadata(path: &Path) -> Result<crate::song_library::models::SongMetadata, ParseError>;
}

pub struct MidiFileParser;

impl MidiParser for MidiFileParser {
    fn parse_metadata(path: &Path) -> Result<crate::song_library::models::SongMetadata, ParseError> {
        let midi = midi_file::MidiFile::new(path).map_err(ParseError::MidiError)?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let duration_secs = midi
            .measures
            .last()
            .map(|d| d.as_secs())
            .unwrap_or(0) as u32;

        let track_count = midi.tracks.len();
        let note_count = midi.tracks.iter().map(|t| t.notes.len()).sum();
        let tempo_changes = midi.tempo_track.events.len();

        Ok(crate::song_library::models::SongMetadata {
            name,
            duration_secs,
            track_count,
            note_count,
            tempo_changes,
        })
    }
}
