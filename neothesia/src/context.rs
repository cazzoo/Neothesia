use std::sync::Arc;

use crate::{
    NeothesiaEvent, TransformUniform, config::Config, input_manager::InputManager,
    output_manager::OutputManager, utils::window::WindowState,
    song_library::{SongRepository, SongLibraryDatabase, default_db_path, Error as SongLibraryError},
};
use neothesia_core::render::{QuadRendererFactory, TextRendererFactory};
use wgpu_jumpstart::{Gpu, Uniform};
use winit::event_loop::EventLoopProxy;

use winit::window::Window;

pub struct Context {
    pub window: Arc<Window>,

    pub window_state: WindowState,
    pub gpu: Gpu,

    pub transform: Uniform<TransformUniform>,
    pub text_renderer_factory: TextRendererFactory,
    pub quad_renderer_facotry: QuadRendererFactory,

    pub output_manager: OutputManager,
    pub input_manager: InputManager,
    pub config: Config,
    pub song_library_db: SongLibraryDatabase,

    pub proxy: EventLoopProxy<NeothesiaEvent>,

    /// Last frame timestamp
    pub frame_timestamp: std::time::Instant,

    #[cfg(debug_assertions)]
    pub fps_ticker: neothesia_core::utils::fps_ticker::Fps,
}

impl Drop for Context {
    fn drop(&mut self) {
        self.config.save();
    }
}

impl Context {
    pub fn new(
        window: Arc<Window>,
        window_state: WindowState,
        proxy: EventLoopProxy<NeothesiaEvent>,
        gpu: Gpu,
    ) -> Self {
        let transform_uniform = Uniform::new(
            &gpu.device,
            TransformUniform::default(),
            wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
        );

        let config = Config::new();

        let text_renderer_factory = TextRendererFactory::new(&gpu);
        let quad_renderer_facotry = QuadRendererFactory::new(&gpu, &transform_uniform);

        let song_library_db = SongLibraryDatabase::with_default_path()
            .unwrap_or_else(|e| {
                log::error!("Failed to initialize song library: {}. Song library features will be disabled.", e);
                SongLibraryDatabase::new(std::path::PathBuf::from("/tmp/neothesia_song_library_disabled.db"))
                    .unwrap_or_else(|_| {
                        log::error!("Completely unable to initialize any song library database");
                        std::process::exit(1);
                    })
            });

        Self {
            window,

            window_state,
            gpu,
            transform: transform_uniform,
            text_renderer_factory,
            quad_renderer_facotry,

            output_manager: Default::default(),
            input_manager: InputManager::new(proxy.clone()),
            config,
            song_library_db,
            proxy,
            frame_timestamp: std::time::Instant::now(),

            #[cfg(debug_assertions)]
            fps_ticker: Default::default(),
        }
    }

    pub fn resize(&mut self) {
        self.transform.data.update(
            self.window_state.physical_size.width as f32,
            self.window_state.physical_size.height as f32,
            self.window_state.scale_factor as f32,
        );
        self.transform.update(&self.gpu.queue);
    }

    pub fn load_song_from_library(&mut self, song_id: i64) -> Option<crate::Song> {
        let entry = self.song_library_db.get_song(song_id).ok()??;

        let midi = midi_file::MidiFile::new(&entry.file_path).ok()?;

        self.config
            .set_last_opened_song(Some(entry.file_path.clone()));
        self.config.save();

        let mut song = crate::Song::new(midi);
        song.song_id = Some(song_id);
        Some(song)
    }

    pub fn refresh_song_library(&self) -> Result<(), SongLibraryError> {
        let song_dirs = self.config.song_directories();
        self.song_library_db.scan_directories(&song_dirs)?;
        Ok(())
    }
}
