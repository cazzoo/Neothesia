use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct SongMetadata {
    pub name: String,
    pub duration_secs: u32,
    pub track_count: usize,
    pub note_count: usize,
    pub tempo_changes: usize,
}

#[derive(Debug, Clone)]
pub struct SongEntry {
    pub id: i64,
    pub file_path: std::path::PathBuf,
    pub name: String,
    pub difficulty: u8,
    pub duration_secs: u32,
    pub track_count: usize,
    pub play_count: u32,
    pub last_score: Option<f32>,
    pub best_score: Option<f32>,
    pub last_played_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SortPreference {
    #[default]
    NameAsc,
    NameDesc,
    DifficultyAsc,
    DifficultyDesc,
    PlayCountDesc,
    PlayCountAsc,
    LastPlayedDesc,
    LastPlayedAsc,
    LastScoreDesc,
    LastScoreAsc,
}

#[derive(Debug, Clone, Default)]
pub struct FilterState {
    pub difficulty_min: Option<u8>,
    pub difficulty_max: Option<u8>,
    pub played_only: bool,
    pub unplayed_only: bool,
    pub search_query: Option<String>,
}

pub fn calculate_difficulty(metadata: &SongMetadata) -> u8 {
    // Handle edge case of zero duration
    let note_density = if metadata.duration_secs > 0 {
        metadata.note_count as f32 / metadata.duration_secs as f32
    } else {
        0.0
    };
    
    let track_factor = metadata.track_count as f32 / 10.0;
    let tempo_factor = (metadata.tempo_changes as f32 / 50.0).min(1.0);
    
    // Weighted score (0-10)
    let score = (note_density / 5.0) * 5.0 + track_factor * 3.0 + tempo_factor * 2.0;
    
    score.clamp(1.0, 10.0) as u8
}

pub fn difficulty_label(difficulty: u8) -> &'static str {
    match difficulty {
        1..=3 => "Easy",
        4..=7 => "Medium",
        8..=10 => "Hard",
        _ => "Unknown",
    }
}
