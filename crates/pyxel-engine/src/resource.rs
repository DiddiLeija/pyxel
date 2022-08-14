use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use chrono::Local;
use platform_dirs::UserDirs;
use zip::write::FileOptions as ZipFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::image::Image;
use crate::music::Music;
use crate::screencast::Screencast;
use crate::settings::{
    NUM_COLORS, NUM_IMAGES, NUM_MUSICS, NUM_SOUNDS, NUM_TILEMAPS, PYXEL_VERSION,
    RESOURCE_ARCHIVE_DIRNAME,
};
use crate::sound::Sound;
use crate::tilemap::Tilemap;
use crate::types::{Color, Rgb8};
use crate::utils::parse_version_string;
use crate::Pyxel;

pub trait ResourceItem {
    fn resource_name(item_no: u32) -> String;
    fn is_modified(&self) -> bool;
    fn clear(&mut self);
    fn serialize(&self, pyxel: &Pyxel) -> String;
    fn deserialize(&mut self, pyxel: &Pyxel, version: u32, input: &str);
}

pub struct Resource {
    capture_scale: u32,
    screencast: Screencast,
}

impl Resource {
    pub fn new(fps: u32, capture_scale: u32, capture_sec: u32) -> Self {
        Self {
            capture_scale: u32::max(capture_scale, 1),
            screencast: Screencast::new(fps, capture_sec),
        }
    }

    pub fn capture_screen(
        &mut self,
        image: &[Vec<Color>],
        colors: &[Rgb8; NUM_COLORS as usize],
        frame_count: u32,
    ) {
        self.screencast.capture(image, colors, frame_count);
    }

    fn export_path() -> String {
        UserDirs::new()
            .unwrap()
            .desktop_dir
            .join(Local::now().format("pyxel-%Y%m%d-%H%M%S").to_string())
            .to_str()
            .unwrap()
            .to_string()
    }
}

impl Pyxel {
    pub fn load(&mut self, filename: &str, image: bool, tilemap: bool, sound: bool, music: bool) {
        let mut archive = ZipArchive::new(
            File::open(&Path::new(&filename))
                .unwrap_or_else(|_| panic!("Unable to open file '{}'", filename)),
        )
        .unwrap_or_else(|_| panic!("Unable to parse zip archive '{}'", filename));
        let version_name = RESOURCE_ARCHIVE_DIRNAME.to_string() + "version";
        let contents = {
            let mut file = archive.by_name(&version_name).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            contents
        };
        let version = parse_version_string(&contents).unwrap();
        assert!(
            version <= parse_version_string(PYXEL_VERSION).unwrap(),
            "Unsupported resource file version '{}'",
            contents
        );

        macro_rules! deserialize {
            ($type: ty, $getter: ident, $count: expr) => {
                for i in 0..$count {
                    if let Ok(mut file) = archive.by_name(&<$type>::resource_name(i)) {
                        let mut input = String::new();
                        file.read_to_string(&mut input).unwrap();
                        self.$getter(i).lock().deserialize(self, version, &input);
                    } else {
                        self.$getter(i).lock().clear();
                    }
                }
            };
        }

        if image {
            deserialize!(Image, image, NUM_IMAGES);
        }
        if tilemap {
            deserialize!(Tilemap, tilemap, NUM_TILEMAPS);
        }
        if sound {
            deserialize!(Sound, sound, NUM_SOUNDS);
        }
        if music {
            deserialize!(Music, music, NUM_MUSICS);
        }
    }

    pub fn save(&mut self, filename: &str, image: bool, tilemap: bool, sound: bool, music: bool) {
        let path = std::path::Path::new(&filename);
        let file = std::fs::File::create(&path)
            .unwrap_or_else(|_| panic!("Unable to open file '{}'", filename));
        let mut zip = ZipWriter::new(file);
        zip.add_directory(RESOURCE_ARCHIVE_DIRNAME, ZipFileOptions::default())
            .unwrap();
        let version_name = RESOURCE_ARCHIVE_DIRNAME.to_string() + "version";
        zip.start_file(version_name, ZipFileOptions::default())
            .unwrap();
        zip.write_all(PYXEL_VERSION.as_bytes()).unwrap();

        macro_rules! serialize {
            ($type: ty, $getter: ident, $count: expr) => {
                for i in 0..$count {
                    if self.$getter(i).lock().is_modified() {
                        zip.start_file(<$type>::resource_name(i), ZipFileOptions::default())
                            .unwrap();
                        zip.write_all(self.$getter(i).lock().serialize(self).as_bytes())
                            .unwrap();
                    }
                }
            };
        }

        if image {
            serialize!(Image, image, NUM_IMAGES);
        }
        if tilemap {
            serialize!(Tilemap, tilemap, NUM_TILEMAPS);
        }
        if sound {
            serialize!(Sound, sound, NUM_SOUNDS);
        }
        if music {
            serialize!(Music, music, NUM_MUSICS);
        }
        zip.finish().unwrap();
    }

    pub fn screenshot(&mut self, scale: Option<u32>) {
        let filename = Resource::export_path();
        let scale = u32::max(scale.unwrap_or(self.resource.capture_scale), 1);
        self.screen.lock().save(&filename, &self.colors, scale);
        self.system.disable_next_frame_skip();
    }

    pub fn reset_capture(&mut self) {
        self.resource.screencast.reset();
    }

    pub fn screencast(&mut self, scale: Option<u32>) {
        let filename = Resource::export_path();
        let scale = u32::max(scale.unwrap_or(self.resource.capture_scale), 1);
        self.resource.screencast.save(&filename, scale);
        self.system.disable_next_frame_skip();
    }
}
