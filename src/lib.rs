#![doc = include_str!("../README.md")]
use std::collections::HashMap;

#[cfg(feature = "ui")]
use bevy::ui::widget::update_image_content_size_system;
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
};
use image::{
    imageops::{self, FilterType},
    GenericImage, GenericImageView, ImageBuffer, ImageError, Rgba,
};
use thiserror::Error;

#[derive(Default)]
pub struct PixelFontPlugin;

impl Plugin for PixelFontPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<PixelFont>()
            .add_systems(PostUpdate, render_sprites.in_set(PixelFontSet))
            .register_type::<PixelFont>()
            .register_type::<PixelFontText>();
        #[cfg(feature = "ui")]
        app.add_systems(
            PostUpdate,
            render_ui_images
                .in_set(PixelFontSet)
                .before(update_image_content_size_system),
        );
    }
}

/// System set for systems related to [`PixelFontPlugin`].
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct PixelFontSet;

/// An image font as well as the mapping of characters to regions inside it.
#[derive(Debug, Clone, Reflect, Default, Asset)]
pub struct PixelFont {
    pub layout: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
    /// The glyph used to render `c` is contained in the part of the image
    /// pointed to by `atlas.textures[index_map[c]]`.
    pub index_map: HashMap<char, usize>,
}

impl PixelFont {
    /// Convenience constructor. The string has newlines (but *not* spaces)
    /// removed, so you can write e.g.
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # let layout = Handle::default();
    /// # let texture = Handle::default();
    /// let chars = r#"
    /// ABCDEFGHIJKLMNOPQR
    /// STUVWXYZ0123456789
    /// "#;
    /// let font = extol_pixel_font::PixelFont::new(layout, texture, chars);
    pub fn new(layout: Handle<TextureAtlasLayout>, texture: Handle<Image>, string: &str) -> Self {
        let chars = string
            .chars()
            .filter(|c| *c != '\n')
            .enumerate()
            .map(|(i, c)| (c, i));
        Self {
            layout,
            texture,
            index_map: chars.collect(),
        }
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
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    for (pixel_font_text, mut image_handle) in &mut query {
        debug!("Rendering [{}]", pixel_font_text.text);
        // don't need to clear the old image since it'll be no longer live
        match render_text(
            &pixel_font_text,
            pixel_fonts.as_ref(),
            images.as_ref(),
            layouts.as_ref(),
        ) {
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
    layouts: Res<Assets<TextureAtlasLayout>>,
) {
    for (pixel_font_text, mut ui_image) in &mut query {
        debug!("Rendering [{}]", pixel_font_text.text);
        // don't need to clear the old image since it'll be no longer live
        match render_text(
            &pixel_font_text,
            pixel_fonts.as_ref(),
            images.as_ref(),
            layouts.as_ref(),
        ) {
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
    #[error("atlas layout asset not loaded")]
    MissingTextureAtlasLayout,
    #[error("internal error")]
    UnknownError,
    #[error("failed to copy from atlas")]
    CopyFailure(#[from] ImageError),
}

/// Renders the text inside the [`PixelFontText`] to a single output image. You
/// don't need to use this if you're using the built-in functionality, but if
/// you want to use this for some other custom plugin/system, you can call this.
pub fn render_text(
    pixel_font_text: &PixelFontText,
    pixel_fonts: &Assets<PixelFont>,
    images: &Assets<Image>,
    layouts: &Assets<TextureAtlasLayout>,
) -> Result<Image, PixelFontPluginError> {
    let pixel_font = pixel_fonts
        .get(&pixel_font_text.font)
        .ok_or(PixelFontPluginError::MissingPixelFontAsset)?;
    let font_texture = images
        .get(&pixel_font.texture)
        .ok_or(PixelFontPluginError::MissingTextureAsset)?;
    let layout = layouts
        .get(&pixel_font.layout)
        .ok_or(PixelFontPluginError::MissingTextureAtlasLayout)?;

    let text = &pixel_font_text.text;

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
