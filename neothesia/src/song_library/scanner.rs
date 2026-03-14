use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct ScanSummary {
    pub songs_added: usize,
    pub songs_updated: usize,
    pub songs_skipped: usize,
    pub errors: Vec<String>,
}

pub trait ProgressCallback: Send + Sync {
    fn on_progress(&self, current: usize, total: usize, message: &str);
    fn on_file_scanned(&self, file: &std::path::Path);
}

pub struct SongScanner {
    parser: Arc<dyn crate::song_library::parser::MidiParser>,
}

impl SongScanner {
    pub fn new() -> Self {
        Self {
            parser: Arc::new(crate::song_library::parser::MidiFileParser),
        }
    }

    pub fn scan_directories(&self, directories: &[PathBuf]) -> Vec<PathBuf> {
        let mut midi_files = Vec::new();
        let mut seen = HashSet::new();

        for dir in directories {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    self.scan_entry(&entry.path(), &mut midi_files, &mut seen);
                }
            }
        }

        midi_files
    }

    fn scan_entry(
        &self,
        path: &std::path::Path,
        midi_files: &mut Vec<PathBuf>,
        seen: &mut HashSet<PathBuf>,
    ) {
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    self.scan_entry(&entry.path(), midi_files, seen);
                }
            }
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("mid") || ext.eq_ignore_ascii_case("midi") {
                    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                    if seen.insert(canonical.clone()) {
                        midi_files.push(canonical);
                    }
                }
            }
        }
    }

    pub fn index_directories(
        &self,
        directories: &[PathBuf],
        repository: &dyn crate::song_library::database::SongRepository,
        progress: Option<&dyn ProgressCallback>,
    ) -> ScanSummary {
        let files = self.scan_directories(directories);
        let total = files.len();
        let mut summary = ScanSummary::default();

        for (idx, path) in files.iter().enumerate() {
            if let Some(prog) = progress {
                prog.on_file_scanned(path);
                prog.on_progress(idx + 1, total, &path.display().to_string());
            }

            match self.parser.parse_metadata(path) {
                Ok(metadata) => {
                    match repository.upsert_song(&metadata, path) {
                        Ok(_) => summary.songs_added += 1,
                        Err(e) => summary.errors.push(format!("{}: {}", path.display(), e)),
                    }
                }
                Err(e) => {
                    summary.errors.push(format!("{}: {}", path.display(), e));
                    summary.songs_skipped += 1;
                }
            }
        }

        summary
    }
}

impl Default for SongScanner {
    fn default() -> Self {
        Self::new()
    }
}
