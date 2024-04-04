/// Demonstrates rendering image font text at both its 'native' height and a
/// scaled-up height.
use bevy::prelude::*;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use extol_image_font::{ImageFont, ImageFontBundle, ImageFontPlugin, ImageFontText};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ImageFontPlugin))
        .init_collection::<DemoAssets>()
        .add_systems(Startup, spawn_text)
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .run();
}

#[derive(AssetCollection, Resource)]
struct DemoAssets {
    #[asset(path = "example_font.image_font.ron")]
    image_font: Handle<ImageFont>,
}

fn spawn_text(mut commands: Commands, assets: Res<DemoAssets>) {
    commands.spawn(Camera2dBundle::default());

    // XXX: shouldn't be exactly on integer coordinates. not sure why.
    commands.spawn(ImageFontBundle {
        text: ImageFontText {
            text: "Sphinx of black quartz, judge my vow!".into(),
            font: assets.image_font.clone(),
            font_height: Some(36.0),
        },
        transform: Transform::from_translation(Vec3::new(0.2, 0.2, 0.2)),
        ..default()
    });
    commands.spawn(ImageFontBundle {
        text: ImageFontText {
            text: "Sphinx of black quartz, judge my vow!".into(),
            font: assets.image_font.clone(),
            font_height: None,
        },
        transform: Transform::from_translation(Vec3::new(0.2, 40.2, 0.2)),
        ..default()
    });
}
