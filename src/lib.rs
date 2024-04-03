#![doc = include_str!("../README.md")]
use std::path::PathBuf;

#[cfg(feature = "ui")]
use bevy::ui::widget::update_image_content_size_system;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext, LoadDirectError},
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    utils::{BoxedFuture, HashMap, HashSet},
};
use image::{
    imageops::{self, FilterType},
    GenericImage, GenericImageView, ImageBuffer, ImageError, Rgba,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Default)]
pub struct PixelFontPlugin;

impl Plugin for PixelFontPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<PixelFont>()
            .add_systems(
                PostUpdate,
                (mark_changed_fonts_as_dirty, render_sprites)
                    .chain()
                    .in_set(PixelFontSet),
            )
            .init_asset_loader::<PixelFontLoader>()
            .register_type::<PixelFont>()
            .register_type::<PixelFontText>();
        #[cfg(feature = "ui")]
        app.add_systems(
            PostUpdate,
            render_ui_images
                .in_set(PixelFontSet)
                .before(update_image_content_size_system)
                .after(mark_changed_fonts_as_dirty),
        );
    }
}

/// System set for systems related to [`PixelFontPlugin`].
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct PixelFontSet;

/// An image font as well as the mapping of characters to regions inside it.
#[derive(Debug, Clone, Reflect, Asset)]
pub struct PixelFont {
    pub layout: TextureAtlasLayout,
    pub texture: Handle<Image>,
    /// The glyph used to render `c` is contained in the part of the image
    /// pointed to by `atlas.textures[index_map[c]]`.
    pub index_map: HashMap<char, usize>,
}

impl PixelFont {
    fn from_char_map(texture: Handle<Image>, size: UVec2, char_map: &HashMap<char, Rect>) -> Self {
        let mut index_map = HashMap::new();
        let mut layout = TextureAtlasLayout::new_empty(size.as_vec2());
        for (i, (&c, &rect)) in char_map.iter().enumerate() {
            index_map.insert(c, i);
            layout.add_texture(rect);
        }
        Self {
            layout,
            texture,
            index_map,
        }
    }

    fn filter_string(&self, s: impl AsRef<str>) -> String {
        s.as_ref()
            .chars()
            .filter(|c| self.index_map.contains_key(c))
            .collect()
    }
}

/// Text rendered using a [`PixelFont`].
#[derive(Debug, Clone, Reflect, Default, Component)]
pub struct PixelFontText {
    pub text: String,
    pub font: Handle<PixelFont>,
    /// If set, overrides the height the font is rendered at. This should be an
    /// integer multiple of the 'native' height if you want pixel accuracy,
    /// but we allow float values for things like animations.
    pub font_height: Option<f32>,
}

/// All the components you need to actually render some text using
/// `extol_pixel_font`.
///
/// NOTE: using exact integer coordinates for the transform can sometimes cause
/// slight rendering issues. I'm not sure why.
#[derive(Bundle, Default)]
pub struct PixelFontBundle {
    pub text: PixelFontText,
    /// Can be used to set the anchor, flip_x, flip_y, etc. Note that the
    /// custom_size property will be recalculated based on
    /// `PixelFontText::font_height`.
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    /// The text will be rendered to this, so you don't need to initialize it.
    pub texture: Handle<Image>,
}

/// System that renders each [`PixelFontText`] into the corresponding
/// `Handle<Image>`. This is mainly for use with sprites.
pub fn render_sprites(
    mut query: Query<(&PixelFontText, &mut Handle<Image>), Changed<PixelFontText>>,
    pixel_fonts: Res<Assets<PixelFont>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (pixel_font_text, mut image_handle) in &mut query {
        debug!("Rendering [{}]", pixel_font_text.text);
        // don't need to clear the old image since it'll be no longer live
        match render_text(&pixel_font_text, pixel_fonts.as_ref(), images.as_ref()) {
            Ok(image) => {
                *image_handle = images.add(image);
            }
            Err(e) => {
                error!(
                    "Error when rendering pixel font text {:?}: {}",
                    pixel_font_text, e
                );
            }
        }
    }
}

#[cfg(feature = "ui")]
/// System that renders each [`PixelFontText`] into the corresponding
/// [`UiImage`].
pub fn render_ui_images(
    mut query: Query<(&PixelFontText, &mut UiImage), Changed<PixelFontText>>,
    pixel_fonts: Res<Assets<PixelFont>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (pixel_font_text, mut ui_image) in &mut query {
        debug!("Rendering [{}]", pixel_font_text.text);
        // don't need to clear the old image since it'll be no longer live
        match render_text(&pixel_font_text, pixel_fonts.as_ref(), images.as_ref()) {
            Ok(image) => {
                ui_image.texture = images.add(image);
            }
            Err(e) => {
                error!(
                    "Error when rendering pixel font text {:?}: {}",
                    pixel_font_text, e
                );
            }
        }
    }
}

/// Errors that can show up during rendering.
#[derive(Debug, Error)]
pub enum PixelFontPluginError {
    #[error("failed to convert image to DynamicImage: {0}")]
    ImageConversion(String),
    #[error("PixelFont asset not loaded")]
    MissingPixelFontAsset,
    #[error("Font texture asset not loaded")]
    MissingTextureAsset,
    #[error("internal error")]
    UnknownError,
    #[error("failed to copy from atlas")]
    CopyFailure(#[from] ImageError),
    #[error("couldn't parse on-disk representation")]
    ParseFailure(#[from] ron::error::SpannedError),
    #[error("i/o error when loading pixel font")]
    Io(#[from] std::io::Error),
    #[error("failed to load asset")]
    LoadDirect(#[from] LoadDirectError),
    #[error("other error {0}")]
    Other(String),
}

/// Renders the text inside the [`PixelFontText`] to a single output image. You
/// don't need to use this if you're using the built-in functionality, but if
/// you want to use this for some other custom plugin/system, you can call this.
pub fn render_text(
    pixel_font_text: &PixelFontText,
    pixel_fonts: &Assets<PixelFont>,
    images: &Assets<Image>,
) -> Result<Image, PixelFontPluginError> {
    let pixel_font = pixel_fonts
        .get(&pixel_font_text.font)
        .ok_or(PixelFontPluginError::MissingPixelFontAsset)?;
    let font_texture = images
        .get(&pixel_font.texture)
        .ok_or(PixelFontPluginError::MissingTextureAsset)?;
    let layout = &pixel_font.layout;

    let text = pixel_font.filter_string(&pixel_font_text.text);

    if text.is_empty() {
        return Ok(Image::new(
            Extent3d {
                width: 0,
                height: 0,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        ));
    }

    // as wide as the sum of all characters, as tall as the tallest one
    let height = text
        .chars()
        .map(|c| layout.textures[pixel_font.index_map[&c]].height())
        .reduce(f32::max)
        .unwrap()
        .ceil() as u32;
    let width = text
        .chars()
        .map(|c| layout.textures[pixel_font.index_map[&c]].width())
        .reduce(|a, b| a + b)
        .unwrap()
        .ceil() as u32;

    let mut output_image = image::RgbaImage::new(width, height);
    let font_texture: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
        font_texture.width(),
        font_texture.height(),
        font_texture.data.as_slice(),
    )
    .ok_or(PixelFontPluginError::UnknownError)?;

    let mut x = 0;
    for c in text.chars() {
        let rect = layout.textures[pixel_font.index_map[&c]];
        let width = rect.width().ceil() as u32;
        let height = rect.height().ceil() as u32;
        output_image.copy_from(
            &*font_texture.view(rect.min.x as u32, rect.min.y as u32, width, height),
            x,
            0,
        )?;
        x += width;
    }

    if let Some(font_height) = pixel_font_text.font_height {
        let width = output_image.width() as f32 * font_height / output_image.height() as f32;
        output_image = imageops::resize(
            &output_image,
            width as u32,
            font_height as u32,
            FilterType::Nearest,
        );
    }

    let mut bevy_image = Image::new(
        Extent3d {
            // these might have changed because of the resize
            width: output_image.width(),
            height: output_image.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        output_image.into_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    bevy_image.sampler = ImageSampler::nearest();
    Ok(bevy_image)
}

pub fn mark_changed_fonts_as_dirty(
    mut events: EventReader<AssetEvent<PixelFont>>,
    mut query: Query<&mut PixelFontText>,
) {
    let changed_fonts: HashSet<_> = events
        .read()
        .filter_map(|event| match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => {
                info!("Pixel font {id} finished loading; marking as dirty");
                Some(id)
            }
            _ => None,
        })
        .collect();
    for mut pixel_font_text in &mut query {
        if changed_fonts.contains(&pixel_font_text.font.id()) {
            pixel_font_text.set_changed();
        }
    }
}

/// Human-readable way to specify where the characters in a pixel font are.
#[derive(Serialize, Deserialize)]
pub enum PixelFontLayout {
    /// Interprets the string as a "grid" and slices up the input image
    /// accordingly. Leading and trailing newlines are stripped, but spaces
    /// are not (since your font might use them as padding).
    Automatic(String),
}

impl PixelFontLayout {
    /// Given the image size, returns a map from each codepoint to its location.
    fn to_char_map(self, size: UVec2) -> HashMap<char, Rect> {
        match self {
            PixelFontLayout::Automatic(str) => {
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
        }
    }
}

/// On-disk representation of a PixelFont, optimized to make it easy for humans
/// to write these.
#[derive(Serialize, Deserialize)]
pub struct PixelFontDiskFormat {
    pub image: PathBuf,
    pub layout: PixelFontLayout,
}

#[derive(Debug, Default)]
pub struct PixelFontLoader;

impl AssetLoader for PixelFontLoader {
    type Asset = PixelFont;

    // We could use PixelFontDiskFormat, but this type has to imnplement `Default`,
    // and there's no sensible default value for that type.
    type Settings = ();

    type Error = PixelFontPluginError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut str = String::new();
            reader.read_to_string(&mut str).await?;
            let disk_format: PixelFontDiskFormat = ron::from_str(&str)?;

            // need the image loaded immediately because we need its size
            let image = load_context
                .load_direct(disk_format.image)
                .await?
                .take::<Image>()
                .ok_or(PixelFontPluginError::Other(
                    "loaded asset wasn't an image".into(),
                ))?;

            let size = image.size();
            let char_map = disk_format.layout.to_char_map(size);
            let image_handle = load_context.add_labeled_asset("texture".into(), image);

            Ok(PixelFont::from_char_map(image_handle, size, &char_map))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pixel_font.ron"]
    }
}
