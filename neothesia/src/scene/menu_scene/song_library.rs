use nuon::TextJustify;
use std::hash::Hash;
use std::sync::Arc;

use crate::{context::Context, song::Song, utils::BoxFuture};
use crate::song_library::{SongEntry, difficulty_label, SongRepository};

use super::{MsgFn, on_async, icons, neo_btn_icon, UiState};

impl super::MenuScene {
    pub fn song_library_page_ui(&mut self, ctx: &mut Context, ui: &mut nuon::Ui) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        let bottom_bar_h = 60.0;

        nuon::translate().x(0.0).y(win_h).build(ui, |ui| {
            let padding = 10.0;
            let w = 80.0;
            let h = bottom_bar_h;

            nuon::translate().y(-padding).add_to_current(ui);
            nuon::translate().y(-h).add_to_current(ui);

            nuon::translate().x(0.0).build(ui, |ui| {
                nuon::translate().x(padding).add_to_current(ui);

                if neo_btn_icon(ui, w, h, icons::left_arrow_icon()) {
                    self.state.go_back();
                }

                nuon::translate().x(-w - padding).add_to_current(ui);
            });

            // Refresh button
            nuon::translate().x(win_w).build(ui, |ui| {
                nuon::translate().x(-w - padding).add_to_current(ui);

                if neo_btn_icon(ui, w, h, icons::repeat_icon()) {
                    if let Err(e) = ctx.refresh_song_library() {
                        log::error!("Failed to refresh song library: {}", e);
                    }
                    self.state.refresh_song_library(&ctx.song_library_db);
                }

                nuon::translate().x(-w - padding).add_to_current(ui);
            });
        });

        let margin_top = 20.0;
        let card_w = 300.0;
        let card_h = 180.0;
        let gap = 16.0;

        let columns = ((win_w - 64.0) / (card_w + gap)).floor().max(1.0) as usize;

        self.song_library_scroll = nuon::scroll()
            .scissor_size(win_w, (win_h - bottom_bar_h).max(0.0))
            .scroll(self.song_library_scroll)
            .build(ui, |ui| {
                if self.state.is_loading {
                    nuon::translate()
                        .x(nuon::center_x(win_w, 200.0))
                        .y(margin_top + 100.0)
                        .build(ui, |ui| {
                            nuon::label()
                                .text("Loading...")
                                .size(200.0, 40.0)
                                .font_size(24.0)
                                .build(ui);
                        });
                } else if self.state.song_library_entries.is_empty() {
                    let content_w = 400.0;
                    let _content_h = 300.0;

                    nuon::translate()
                        .x(nuon::center_x(win_w, content_w))
                        .y(margin_top + 50.0)
                        .build(ui, |ui| {
                            nuon::label()
                                .text("📚")
                                .size(content_w, 80.0)
                                .font_size(64.0)
                                .build(ui);

                            nuon::translate().y(80.0).add_to_current(ui);

                            nuon::label()
                                .text("No songs found")
                                .size(content_w, 40.0)
                                .font_size(24.0)
                                .build(ui);

                            nuon::translate().y(40.0).add_to_current(ui);

                            nuon::label()
                                .text("Add song directories in Settings to build your library.")
                                .size(content_w, 30.0)
                                .font_size(14.0)
                                .color([150, 150, 150])
                                .build(ui);

                            nuon::translate().y(50.0).add_to_current(ui);

                            let btn_w = 160.0;
                            let btn_h = 44.0;
                            if nuon::button()
                                .x((content_w - btn_w) / 2.0)
                                .size(btn_w, btn_h)
                                .color([80, 150, 200])
                                .hover_color([100, 170, 220])
                                .border_radius([7.0; 4])
                                .label("Go to Settings")
                                .build(ui)
                            {
                                self.state.go_to(super::state::Page::Settings);
                            }
                        });
                } else {
                    nuon::translate()
                        .x(nuon::center_x(win_w, columns as f32 * (card_w + gap) - gap))
                        .y(margin_top)
                        .add_to_current(ui);

                    let total_entries = self.state.song_library_entries.len();
                    let mut entry_idx = 0;

                    loop {
                        if entry_idx >= total_entries {
                            break;
                        }
                        
                        let row_end = (entry_idx + columns).min(total_entries);

                        // Clone entries for this row before entering the closure
                        let row_entries: Vec<SongEntry> = self.state.song_library_entries[entry_idx..row_end].iter().cloned().collect();
                        entry_idx = row_end;

                        nuon::translate().build(ui, |ui| {
                            for entry in row_entries {
                                self.song_card(&*ctx, ui, &entry, card_w, card_h);
                                nuon::translate().x(card_w + gap).add_to_current(ui);
                            }
                        });

                        nuon::translate().y(card_h + gap).add_to_current(ui);
                    }
                }
            });
    }

    fn song_card(&mut self, _ctx: &Context, ui: &mut nuon::Ui, entry: &SongEntry, w: f32, h: f32) {
        let pad = 16.0;

        nuon::quad()
            .size(w, h)
            .color([37, 35, 42])
            .border_radius([12.0; 4])
            .build(ui);

        let click_id = nuon::Id::hash_with(|h| {
            "song_card".hash(h);
            entry.id.hash(h);
        });

        let click_event = nuon::click_area(click_id).size(w, h).build(ui);

        if click_event.is_clicked() {
            self.futures.push(load_song_from_library(entry.id, &mut self.state));
        }

        nuon::translate().pos(pad, pad).build(ui, |ui| {
            let inner_w = w - pad * 2.0;

            nuon::label()
                .size(inner_w, 30.0)
                .text(&entry.name)
                .font_size(18.0)
                .text_justify(TextJustify::Left)
                .build(ui);

            nuon::translate().y(30.0).add_to_current(ui);

            let difficulty = difficulty_label(entry.difficulty);
            let color = match entry.difficulty {
                1..=3 => [80, 180, 112],
                4..=7 => [180, 168, 80],
                8..=10 => [180, 80, 80],
                _ => [150, 150, 150],
            };

            nuon::label()
                .size(inner_w, 20.0)
                .text(&format!("Difficulty: {}", difficulty))
                .font_size(14.0)
                .color(color)
                .text_justify(TextJustify::Left)
                .build(ui);

            nuon::translate().y(20.0).add_to_current(ui);

            let play_text = if entry.play_count == 0 {
                "Never played".to_string()
            } else {
                format!("Played {} times", entry.play_count)
            };

            nuon::label()
                .size(inner_w, 18.0)
                .text(&play_text)
                .font_size(12.0)
                .color([150, 150, 150])
                .text_justify(TextJustify::Left)
                .build(ui);

            nuon::translate().y(18.0).add_to_current(ui);

            if let Some(score) = entry.last_score {
                nuon::label()
                    .size(inner_w, 18.0)
                    .text(&format!("Last Score: {:.0}%", score))
                    .font_size(12.0)
                    .color([150, 150, 150])
                    .text_justify(TextJustify::Left)
                    .build(ui);

                nuon::translate().y(18.0).add_to_current(ui);
            }

            if let Some(best) = entry.best_score {
                nuon::label()
                    .size(inner_w, 18.0)
                    .text(&format!("Best Score: {:.0}%", best))
                    .font_size(12.0)
                    .color([150, 200, 150])
                    .text_justify(TextJustify::Left)
                    .build(ui);
            }
        });

        if click_event.is_hovered() {
            nuon::quad()
                .size(w, h)
                .color([255, 255, 255, 10])
                .border_radius([12.0; 4])
                .build(ui);
        }
    }
}

fn load_song_from_library(id: i64, data: &mut UiState) -> BoxFuture<MsgFn> {
    data.is_loading = true;
    on_async(load_song_from_library_fut(id), move |res, data, ctx| {
        if let Some((midi, song_id, file_path)) = res {
            ctx.config
                .set_last_opened_song(Some(file_path.clone()));
            ctx.config.save();
            
            let mut song = Song::new(midi);
            song.song_id = Some(song_id);
            data.song = Some(song);
        }
        data.is_loading = false;
    })
}

async fn load_song_from_library_fut(id: i64) -> Option<(midi_file::MidiFile, i64, std::path::PathBuf)> {
    let db = crate::song_library::init_song_library().ok()?;
    let entry = db.get_song(id).ok()??;
    let file_path = entry.file_path.clone();
    let file_path_for_return = file_path.clone();

    let thread = crate::utils::task::thread::spawn("midi-loader".into(), move || {
        midi_file::MidiFile::new(&file_path).ok()
    });

    let midi_file = thread.join().await.ok().flatten()?;
    Some((midi_file, id, file_path_for_return))
}
