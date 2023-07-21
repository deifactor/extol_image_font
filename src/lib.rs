use std::collections::HashMap;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{Extract, RenderApp},
    sprite::{
        extract_sprites, queue_sprites, Anchor, ExtractedSprite, ExtractedSprites, SpriteSystem,
    },
};

/// Plugin that enables rendering fonts.
#[derive(Default)]
pub struct PixelFontPlugin;

impl Plugin for PixelFontPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<PixelFont>()
            .add_systems(Update, update_pixel_font_layout);
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            ExtractSchedule,
            extract_text_sprite
                .in_set(SpriteSystem::ExtractSprites)
                .after(extract_sprites)
                .before(queue_sprites),
        );
    }
}

/// An image font as well as the mapping of characters to regions inside it.
#[derive(TypeUuid, Debug, Clone, Reflect, Default)]
#[uuid = "5f2e937e-cb24-4642-a26d-512c77bf4459"]
pub struct PixelFont {
    pub atlas: Handle<TextureAtlas>,
    /// The glyph used to render `c` is contained in the part of the image
    /// pointed to by `atlas.textures[index_map[c]]`.
    index_map: HashMap<char, usize>,
}

impl PixelFont {
    /// Convenience constructor. The string has newlines (but *not* spaces)
    /// removed, so you can write e.g.
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # let atlas = Handle::default();
    /// let chars = r#"
    /// ABCDEFGHIJKLMNOPQR
    /// STUVWXYZ0123456789
    /// "#;
    /// let font = extol_pixel_font::PixelFont::new(atlas, chars);
    pub fn new(atlas: Handle<TextureAtlas>, string: &str) -> Self {
        let chars = string
            .chars()
            .filter(|c| *c != '\n')
            .enumerate()
            .map(|(i, c)| (c, i));
        Self {
            atlas,
            index_map: chars.collect(),
        }
    }
}

/// Text rendered using an [`PixelFont`].
#[derive(Debug, Clone, Reflect, Default, Component)]
pub struct PixelFontText {
    pub text: String,
    pub font: Handle<PixelFont>,
    /// If set, overrides the height the font is rendered at. This should be an
    /// integer multiple of the 'native' height, but we allow float values for
    /// things like animations.
    pub font_height: Option<f32>,
}

/// Layout information about an [`PixelFontText`]. This is computed whenever the
/// [`PixelFontText`] is updated or created, so you don't need to manually
/// manage this.
#[derive(Debug, Clone, Reflect, Default, Component)]
pub struct TextLayout {
    size: Vec2,
    glyphs: Vec<Glyph>,
}

/// A single symbol inside a piece of rendered text.
#[derive(Debug, Clone, Reflect, Default)]
struct Glyph {
    /// Position relative to the entire text string (i.e., not the position
    /// inside the atlas).
    position: Vec2,
    /// Size of this individual glyph. Might differ from the usual if the font
    /// is scaled.
    size: Vec2,
    atlas_index: usize,
}

/// All the components you need to actually render some text using
/// `extol_pixel_font`.
#[derive(Bundle, Default)]
pub struct PixelFontBundle {
    pub text: PixelFontText,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    /// Where the transform point is located relative to the entire string; does
    /// not affect the position of individual letters within the string.
    pub anchor: Anchor,
    /// Automatically computed; you can leave this default-initialized.
    pub layout: TextLayout,
}

/// Update all [`TextLayout`]s whose [`PixelFontText`] has changed since this
/// system last ran.
#[allow(clippy::type_complexity)]
pub fn update_pixel_font_layout(
    fonts: Res<Assets<PixelFont>>,
    atlases: Res<Assets<TextureAtlas>>,
    mut text_query: Query<(&PixelFontText, &mut TextLayout), Changed<PixelFontText>>,
) {
    for (text, mut layout) in &mut text_query {
        let Some(font) = fonts.get(&text.font) else { continue; };
        let Some(atlas) = atlases.get(&font.atlas) else { continue; };

        let mut size = Vec2::ZERO;
        let mut position = Vec2::ZERO;
        let glyphs: Vec<Glyph> = text
            .text
            .chars()
            .filter_map(|c| {
                let atlas_index = *font.index_map.get(&c)?;
                assert!(c != '\n', "newlines are not yet supported");

                let rect = atlas.textures[atlas_index];
                let scale = text
                    .font_height
                    .map_or(1.0, |height| height / rect.height());

                let glyph = Glyph {
                    position,
                    size: rect.size() * scale,
                    atlas_index,
                };
                position += Vec2::X * rect.width() * scale;
                size = rect.size() * scale + position;
                Some(glyph)
            })
            .collect();
        *layout = TextLayout { size, glyphs };
    }
}

#[allow(clippy::type_complexity)]
pub fn extract_text_sprite(
    mut extracted_sprites: ResMut<ExtractedSprites>,
    fonts: Extract<Res<Assets<PixelFont>>>,
    texture_atlases: Extract<Res<Assets<TextureAtlas>>>,
    text_query: Extract<
        Query<(
            Entity,
            &ComputedVisibility,
            &GlobalTransform,
            &PixelFontText,
            &TextLayout,
            &Anchor,
        )>,
    >,
) {
    for (entity, visibility, text_transform, text, text_layout, anchor) in &text_query {
        if !visibility.is_visible() {
            continue;
        }
        let Some(font) = fonts.get(&text.font) else { continue; };
        let Some(atlas) = texture_atlases.get(&font.atlas) else { continue; };
        let image_handle_id = atlas.texture.clone_weak().id();
        let alignment_translation = text_layout.size * (-anchor.as_vec() - 0.5);
        for glyph in &text_layout.glyphs {
            let transform = *text_transform
                * Transform::from_translation(alignment_translation.extend(0.))
                * Transform::from_translation(glyph.position.extend(0.));
            let rect = atlas.textures[glyph.atlas_index];
            extracted_sprites.sprites.push(ExtractedSprite {
                entity,
                transform,
                color: Color::default(),
                rect: Some(rect),
                custom_size: Some(glyph.size),
                image_handle_id,
                flip_x: false,
                flip_y: false,
                anchor: Anchor::Center.as_vec(),
            });
        }
    }
}
