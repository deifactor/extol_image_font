//! An example that demonstrates the effect of y-sorting. The two sets of
//! squares have the same coordinates, but the one on the right uses sprite
//! layers and so is y-sorted. Tap space to toggle y-sorting.
use bevy::{prelude::*, sprite::Anchor};
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use extol_pixel_font::{PixelFont, PixelFontBundle, PixelFontPlugin, PixelFontText};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(ImagePlugin::default_nearest()),
            PixelFontPlugin,
        ))
        .insert_resource(Msaa::Off)
        .init_collection::<DemoAssets>()
        .add_systems(Startup, spawn_text)
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
    mut pixel_fonts: ResMut<Assets<PixelFont>>,
) {
    let s = r##"
 !"#$%&'()*+,-./0123
456789:;<=>?@ABCDEFG
HIJKLMNOPQRSTUVWXYZ[
\]^_`abcdefghijklmno
pqrstuvwxyz{|}~
"##;
    let pixel_font = pixel_fonts.add(PixelFont::new(assets.sprite.clone(), s));
    commands.spawn(Camera2dBundle::default());

    // XXX: shouldn't be exactly on integer coordinates. not sure why.
    commands.spawn(PixelFontBundle {
        text: PixelFontText {
            text: "Sphinx of black quartz, judge my vow!".into(),
            font: pixel_font.clone(),
            font_height: Some(36.0),
        },
        transform: Transform::from_translation(Vec3::new(0.2, 0.2, 0.2)),
        anchor: Anchor::Center,
        ..default()
    });
    commands.spawn(PixelFontBundle {
        text: PixelFontText {
            text: "Sphinx of black quartz, judge my vow!".into(),
            font: pixel_font,
            font_height: None,
        },
        transform: Transform::from_translation(Vec3::new(0.2, 40.2, 0.2)),
        anchor: Anchor::Center,
        ..default()
    });
}
