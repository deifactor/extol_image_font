#![doc = include_str!("../README.md")]

pub mod loader;

#[cfg(feature = "ui")]
use bevy::ui::widget::update_image_content_size_system;
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    utils::{HashMap, HashSet},
};
use image::{
    imageops::{self, FilterType},
    GenericImage, GenericImageView, ImageBuffer, ImageError, Rgba,
};
use thiserror::Error;

#[derive(Default)]
pub struct ImageFontPlugin;

impl Plugin for ImageFontPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ImageFont>()
            .add_systems(
                PostUpdate,
                (mark_changed_fonts_as_dirty, render_sprites)
                    .chain()
                    .in_set(ImageFontSet),
            )
            .init_asset_loader::<loader::ImageFontLoader>()
            .register_type::<ImageFont>()
            .register_type::<ImageFontText>();
        #[cfg(feature = "ui")]
        app.add_systems(
            PostUpdate,
            render_ui_images
                .in_set(ImageFontSet)
                .before(update_image_content_size_system)
                .after(mark_changed_fonts_as_dirty),
        );
    }
}

/// System set for systems related to [`ImageFontPlugin`].
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct ImageFontSet;

/// An image font as well as the mapping of characters to regions inside it.
#[derive(Debug, Clone, Reflect, Asset)]
pub struct ImageFont {
    pub layout: TextureAtlasLayout,
    pub texture: Handle<Image>,
    /// The glyph used to render `c` is contained in the part of the image
    /// pointed to by `atlas.textures[index_map[c]]`.
    pub index_map: HashMap<char, usize>,
}

impl ImageFont {
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

/// Text rendered using an [`ImageFont`].
#[derive(Debug, Clone, Reflect, Default, Component)]
pub struct ImageFontText {
    pub text: String,
    pub font: Handle<ImageFont>,
    /// If set, overrides the height the font is rendered at. This should be an
    /// integer multiple of the 'native' height if you want pixel accuracy,
    /// but we allow float values for things like animations.
    pub font_height: Option<f32>,
}

/// All the components you need to actually render some text using
/// `extol_image_font`.
///
/// NOTE: using exact integer coordinates for the transform can sometimes cause
/// slight rendering issues. I'm not sure why.
#[derive(Bundle, Default)]
pub struct ImageFontBundle {
    pub text: ImageFontText,
    /// Can be used to set the anchor, flip_x, flip_y, etc. Note that the
    /// custom_size property will be recalculated based on
    /// `ImageFontText::font_height`.
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    /// The text will be rendered to this, so you don't need to initialize it.
    pub texture: Handle<Image>,
}

/// System that renders each [`ImageFontText`] into the corresponding
/// `Handle<Image>`. This is mainly for use with sprites.
pub fn render_sprites(
    mut query: Query<(&ImageFontText, &mut Handle<Image>), Changed<ImageFontText>>,
    image_fonts: Res<Assets<ImageFont>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (image_font_text, mut image_handle) in &mut query {
        debug!("Rendering [{}]", image_font_text.text);
        // don't need to clear the old image since it'll be no longer live
        match render_text(image_font_text, image_fonts.as_ref(), images.as_ref()) {
            Ok(image) => {
                *image_handle = images.add(image);
            }
            Err(e) => {
                error!(
                    "Error when rendering image font text {:?}: {}",
                    image_font_text, e
                );
            }
        }
    }
}

#[cfg(feature = "ui")]
/// System that renders each [`ImageFontText`] into the corresponding
/// [`UiImage`].
pub fn render_ui_images(
    mut query: Query<(&ImageFontText, &mut UiImage), Changed<ImageFontText>>,
    image_fonts: Res<Assets<ImageFont>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (image_font_text, mut ui_image) in &mut query {
        debug!("Rendering [{}]", image_font_text.text);
        // don't need to clear the old image since it'll be no longer live
        match render_text(image_font_text, image_fonts.as_ref(), images.as_ref()) {
            Ok(image) => {
                ui_image.texture = images.add(image);
            }
            Err(e) => {
                error!(
                    "Error when rendering image font text {:?}: {}",
                    image_font_text, e
                );
            }
        }
    }
}

/// Errors that can show up during rendering.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ImageFontRenderError {
    #[error("failed to convert image to DynamicImage: {0}")]
    ImageConversion(String),
    #[error("ImageFont asset not loaded")]
    MissingImageFontAsset,
    #[error("Font texture asset not loaded")]
    MissingTextureAsset,
    #[error("internal error")]
    UnknownError,
    #[error("failed to copy from atlas")]
    CopyFailure(#[from] ImageError),
}

/// Renders the text inside the [`ImageFontText`] to a single output image. You
/// don't need to use this if you're using the built-in functionality, but if
/// you want to use this for some other custom plugin/system, you can call this.
#[allow(clippy::result_large_err)]
pub fn render_text(
    image_font_text: &ImageFontText,
    image_fonts: &Assets<ImageFont>,
    images: &Assets<Image>,
) -> Result<Image, ImageFontRenderError> {
    let image_font = image_fonts
        .get(&image_font_text.font)
        .ok_or(ImageFontRenderError::MissingImageFontAsset)?;
    let font_texture = images
        .get(&image_font.texture)
        .ok_or(ImageFontRenderError::MissingTextureAsset)?;
    let layout = &image_font.layout;

    let text = image_font.filter_string(&image_font_text.text);

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
        .map(|c| layout.textures[image_font.index_map[&c]].height())
        .reduce(f32::max)
        .unwrap()
        .ceil() as u32;
    let width = text
        .chars()
        .map(|c| layout.textures[image_font.index_map[&c]].width())
        .reduce(|a, b| a + b)
        .unwrap()
        .ceil() as u32;

    let mut output_image = image::RgbaImage::new(width, height);
    let font_texture: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
        font_texture.width(),
        font_texture.height(),
        font_texture.data.as_slice(),
    )
    .ok_or(ImageFontRenderError::UnknownError)?;

    let mut x = 0;
    for c in text.chars() {
        let rect = layout.textures[image_font.index_map[&c]];
        let width = rect.width().ceil() as u32;
        let height = rect.height().ceil() as u32;
        output_image.copy_from(
            &*font_texture.view(rect.min.x as u32, rect.min.y as u32, width, height),
            x,
            0,
        )?;
        x += width;
    }

    if let Some(font_height) = image_font_text.font_height {
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

/// Marks any text where the underlying [`ImageFont`] asset has changed as
/// dirty, which will cause it to be rerendered.
pub fn mark_changed_fonts_as_dirty(
    mut events: EventReader<AssetEvent<ImageFont>>,
    mut query: Query<&mut ImageFontText>,
) {
    let changed_fonts: HashSet<_> = events
        .read()
        .filter_map(|event| match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => {
                info!("Image font {id} finished loading; marking as dirty");
                Some(id)
            }
            _ => None,
        })
        .collect();
    for mut image_font_text in &mut query {
        if changed_fonts.contains(&image_font_text.font.id()) {
            image_font_text.set_changed();
        }
    }
}
