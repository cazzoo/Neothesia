pub mod database;
pub mod models;
pub mod parser;
pub mod scanner;

pub use database::{SongRepository, SqliteSongRepository, DatabaseError};
pub use models::{SongEntry, SortPreference, FilterState, difficulty_label};
pub use scanner::SongScanner;

// Re-export commonly used types
pub type SongLibraryDatabase = SqliteSongRepository;

// Re-export Error for convenience
pub use database::DatabaseError as Error;

use std::path::PathBuf;

/// Get the default database path
pub fn default_db_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("neothesia").join("song_library.db")
}

/// Initialize the song library with default settings
pub fn init_song_library() -> Result<SongLibraryDatabase, DatabaseError> {
    let db_path = default_db_path();
    
    SongLibraryDatabase::new(db_path)
}
