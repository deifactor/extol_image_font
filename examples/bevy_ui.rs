/// Shows use of the plugin with bevy_ui.
use bevy::prelude::*;
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use extol_pixel_font::{PixelFont, PixelFontPlugin, PixelFontText};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(ImagePlugin::default_nearest()),
            PixelFontPlugin,
        ))
        .insert_resource(Msaa::Off)
        .init_collection::<DemoAssets>()
        .add_systems(Startup, spawn_ui)
        .insert_resource(ClearColor(Color::BLACK))
        .run();
}

#[derive(AssetCollection, Resource)]
struct DemoAssets {
    #[asset(texture_atlas(tile_size_x = 5., tile_size_y = 12., columns = 20, rows = 5))]
    layout: Handle<TextureAtlasLayout>,
    #[asset(path = "example_font.png")]
    sprite: Handle<Image>,
}

fn spawn_ui(
    mut commands: Commands,
    assets: Res<DemoAssets>,
    mut pixel_fonts: ResMut<Assets<PixelFont>>,
) {
    // note that the newlines here are stripped; we use the `TextureAtlasLayout`'s
    // size information.
    let s = r##"
 !"#$%&'()*+,-./0123
456789:;<=>?@ABCDEFG
HIJKLMNOPQRSTUVWXYZ[
\]^_`abcdefghijklmno
pqrstuvwxyz{|}~
"##;
    let pixel_font = pixel_fonts.add(PixelFont::new(
        assets.layout.clone(),
        assets.sprite.clone(),
        s,
    ));
    commands.spawn(Camera2dBundle::default());

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                ImageBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    ..default()
                },
                PixelFontText {
                    text: "Points: 0".into(),
                    font: pixel_font.clone(),
                    font_height: None,
                },
            ));
        });
}
