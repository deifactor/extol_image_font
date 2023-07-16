//! An example that demonstrates the effect of y-sorting. The two sets of
//! squares have the same coordinates, but the one on the right uses sprite
//! layers and so is y-sorted. Tap space to toggle y-sorting.
use bevy::{prelude::*, sprite::Anchor};
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use extol_image_font::{ImageFont, ImageFontBundle, ImageFontPlugin, ImageFontText};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(ImagePlugin::default_nearest()))
        .add_plugin(ImageFontPlugin)
        .init_collection::<DemoAssets>()
        .add_startup_system(spawn_text)
        .insert_resource(ClearColor(Color::BLACK))
        .run();
}

#[derive(AssetCollection, Resource)]
struct DemoAssets {
    #[asset(texture_atlas(tile_size_x = 5., tile_size_y = 12., columns = 20, rows = 5))]
    #[asset(path = "example_font.png")]
    sprite: Handle<TextureAtlas>,
}

fn spawn_text(
    mut commands: Commands,
    assets: Res<DemoAssets>,
    mut image_fonts: ResMut<Assets<ImageFont>>,
) {
    let s = r##"
 !"#$%&'()*+,-./0123
456789:;<=>?@ABCDEFG
HIJKLMNOPQRSTUVWXYZ[
\]^_`abcdefghijklmno
pqrstuvwxyz{|}~
"##;
    let image_font = image_fonts.add(ImageFont::new(assets.sprite.clone(), s));
    commands.spawn(Camera2dBundle::default());
    commands.spawn(ImageFontBundle {
        text: ImageFontText {
            text: "Sphinx of black quartz, judge my vow".into(),
            font: image_font.clone(),
        },
        transform: Transform::from_translation(0.5 * Vec3::ONE).with_scale(1.0 * Vec3::ONE),
        anchor: Anchor::TopLeft,
        ..default()
    });
    commands.spawn(ImageFontBundle {
        text: ImageFontText {
            text: "Sphinx of black quartz, judge my vow!".into(),
            font: image_font,
        },
        transform: Transform::from_translation(0.5 * Vec3::ONE + 40. * Vec3::Y)
            .with_scale(1.0 * Vec3::ONE),
        anchor: Anchor::TopLeft,
        ..default()
    });
}
