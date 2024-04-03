/// Demonstrates rendering pixel font text at both its 'native' height and a
/// scaled-up height.
use bevy::prelude::*;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use extol_pixel_font::{PixelFont, PixelFontBundle, PixelFontPlugin, PixelFontText};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PixelFontPlugin))
        .init_collection::<DemoAssets>()
        .add_systems(Startup, spawn_text)
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .run();
}

#[derive(AssetCollection, Resource)]
struct DemoAssets {
    #[asset(path = "example_font.pixel_font.ron")]
    pixel_font: Handle<PixelFont>,
}

fn spawn_text(mut commands: Commands, assets: Res<DemoAssets>) {
    commands.spawn(Camera2dBundle::default());

    // XXX: shouldn't be exactly on integer coordinates. not sure why.
    commands.spawn(PixelFontBundle {
        text: PixelFontText {
            text: "Sphinx of black quartz, judge my vow!".into(),
            font: assets.pixel_font.clone(),
            font_height: Some(36.0),
        },
        transform: Transform::from_translation(Vec3::new(0.2, 0.2, 0.2)),
        ..default()
    });
    commands.spawn(PixelFontBundle {
        text: PixelFontText {
            text: "Sphinx of black quartz, judge my vow!".into(),
            font: assets.pixel_font.clone(),
            font_height: None,
        },
        transform: Transform::from_translation(Vec3::new(0.2, 40.2, 0.2)),
        ..default()
    });
}
