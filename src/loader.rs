//! Code for parsing an [`ImageFont`] off of an on-disk representation.
use std::path::PathBuf;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext, LoadDirectError},
    prelude::*,
    utils::{BoxedFuture, HashMap},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ImageFont;

/// Human-readable way to specify where the characters in an image font are.
#[derive(Serialize, Deserialize)]
pub enum ImageFontLayout {
    /// Interprets the string as a "grid" and slices up the input image
    /// accordingly. Leading and trailing newlines are stripped, but spaces
    /// are not (since your font might use them as padding).
    ///
    /// ```rust
    /// # use extol_image_font::loader::*;
    /// // note that we have a raw string *inside* a raw string here...
    /// let s = r###"
    ///
    /// // this bit is the actual RON syntax
    /// Automatic(r##"
    ///  !"#$%&'()*+,-./0123
    /// 456789:;<=>?@ABCDEFG
    /// HIJKLMNOPQRSTUVWXYZ[
    /// \]^_`abcdefghijklmno
    /// pqrstuvwxyz{|}~
    /// "##)
    ///
    /// "###;
    /// let layout = ron::from_str::<ImageFontLayout>(s).unwrap();
    /// ```
    Automatic(String),

    /// Manually specifies the top-left position of each character, where each
    /// character has the same size. When writing this in RON, the syntax
    /// will look like
    ///
    /// ```rust
    /// # use extol_image_font::loader::*;
    /// let s = r#"
    /// ManualMonospace(
    ///   size: (4, 8),
    ///   coords: {
    ///      'a': (0, 0),
    ///      'b': (10, 0)
    ///   }
    /// )
    /// "#;
    /// ron::from_str::<ImageFontLayout>(s).unwrap();
    /// ```
    ManualMonospace {
        size: UVec2,
        coords: HashMap<char, UVec2>,
    },

    /// Fully specifies the bounds of each character. The most general case.
    ///
    /// ```rust
    /// # use extol_image_font::loader::*;
    /// let s = r#"
    /// Manual({
    /// 'a': URect(min: (0, 0), max: (10, 20)),
    /// 'b': URect(min: (20, 20), max: (25, 25))
    /// })
    /// "#;
    /// ron::from_str::<ImageFontLayout>(s).unwrap();
    /// ```
    Manual(HashMap<char, URect>),
}

impl ImageFontLayout {
    /// Given the image size, returns a map from each codepoint to its location.
    fn into_char_map(self, size: UVec2) -> HashMap<char, Rect> {
        match self {
            ImageFontLayout::Automatic(str) => {
                // trim() removes whitespace, which is not what we want!
                let str = str.trim_start_matches('\n').trim_end_matches('\n');
                let mut rect_map = HashMap::new();
                let max_chars_per_line = str
                    .lines()
                    // important: *not* l.len()
                    .map(|l| l.chars().count())
                    .max()
                    .expect("can't create character map from an empty string")
                    as u32;

                if size.x % max_chars_per_line != 0 {
                    warn!(
                        "image width {} is not an exact multiple of character count {}",
                        size.x, max_chars_per_line
                    );
                }
                let line_count = str.lines().count() as u32;
                if size.y % line_count != 0 {
                    warn!(
                        "image height {} is not an exact multiple of character count {}",
                        size.y, line_count
                    );
                }

                let rect_width = (size.x / max_chars_per_line) as f32;
                let rect_height = (size.y / line_count) as f32;

                for (row, line) in str.lines().enumerate() {
                    for (col, char) in line.chars().enumerate() {
                        let rect = Rect::new(
                            rect_width * col as f32,
                            rect_height * row as f32,
                            rect_width * (col + 1) as f32,
                            rect_height * (row + 1) as f32,
                        );
                        rect_map.insert(char, rect);
                    }
                }
                rect_map
            }
            ImageFontLayout::ManualMonospace { size, coords } => coords
                .into_iter()
                .map(|(c, top_left)| {
                    (
                        c,
                        Rect::from_corners(top_left.as_vec2(), (size + top_left).as_vec2()),
                    )
                })
                .collect(),
            ImageFontLayout::Manual(urect_map) => urect_map
                .into_iter()
                .map(|(k, v)| (k, v.as_rect()))
                .collect(),
        }
    }
}

/// On-disk representation of a ImageFont, optimized to make it easy for humans
/// to write these. See the docs for [`ImageFontLayout`]'s variants for
/// information on how to write the syntax, or [the example font's RON asset].
///
/// [the example font's RON asset](https://github.com/deifactor/extol_image_font/blob/main/assets/example_font.image_font.ron)
#[derive(Serialize, Deserialize)]
pub struct ImageFontSettings {
    pub image: PathBuf,
    pub layout: ImageFontLayout,
}

/// Loader for [`ImageFont`]s.
#[derive(Debug, Default)]
pub struct ImageFontLoader;

/// Errors that can show up during loading.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ImageFontLoadError {
    #[error("couldn't parse on-disk representation: {0}")]
    ParseFailure(#[from] ron::error::SpannedError),
    #[error("i/o error when loading image font: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to load asset: {0}")]
    LoadDirect(#[from] LoadDirectError),
    #[error("path at {0} wasn't loaded as an image")]
    NotAnImage(PathBuf),
}

impl AssetLoader for ImageFontLoader {
    type Asset = ImageFont;

    // We could use ImageFontSettings, but an AssetLoader's settings has to
    // imnplement `Default`, and there's no sensible default value for that
    // type.
    type Settings = ();

    type Error = ImageFontLoadError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut str = String::new();
            reader.read_to_string(&mut str).await?;
            let disk_format: ImageFontSettings = ron::from_str(&str)?;

            // need the image loaded immediately because we need its size
            let image = load_context
                .load_direct(disk_format.image.clone())
                .await?
                .take::<Image>()
                .ok_or(ImageFontLoadError::NotAnImage(disk_format.image))?;

            let size = image.size();
            let char_map = disk_format.layout.into_char_map(size);
            let image_handle = load_context.add_labeled_asset("texture".into(), image);

            Ok(ImageFont::from_char_map(image_handle, size, &char_map))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["image_font.ron"]
    }
}
