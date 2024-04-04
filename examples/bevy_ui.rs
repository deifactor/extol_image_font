/// Shows use of the plugin with bevy_ui.
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_asset_loader::prelude::{AssetCollection, AssetCollectionApp};
use extol_image_font::{ImageFont, ImageFontPlugin, ImageFontText, ImageFontUiBundle};

#[derive(Default, Debug, Resource)]
struct VowsJudged(u32);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ImageFontPlugin))
        .init_collection::<DemoAssets>()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .init_resource::<VowsJudged>()
        .add_systems(Startup, spawn_ui)
        .add_systems(
            Update,
            (
                judge.run_if(input_just_pressed(KeyCode::Space)),
                update_vows_node,
            )
                .chain(),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct DemoAssets {
    #[asset(path = "example_font.image_font.ron")]
    image_font: Handle<ImageFont>,
}

#[derive(Component)]
struct VowsNode;

fn spawn_ui(mut commands: Commands, assets: Res<DemoAssets>) {
    commands.spawn(Camera2dBundle::default());

    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                ImageBundle {
                    style: Style {
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    ..default()
                },
                ImageFontText::default()
                    .text("Press SPACE to judge!")
                    .font(assets.image_font.clone())
                    .font_height(72.0),
            ));
        });

    commands.spawn((
        VowsNode,
        ImageFontUiBundle {
            node: ImageBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Auto,
                    right: Val::Px(0.0),
                    ..default()
                },
                ..default()
            },
            text: ImageFontText::default()
                .font(assets.image_font.clone())
                .font_height(72.0),
        },
    ));
}

fn judge(mut vows: ResMut<VowsJudged>) {
    vows.0 += 1;
}

fn update_vows_node(vows: Res<VowsJudged>, mut node: Query<&mut ImageFontText, With<VowsNode>>) {
    if vows.is_changed() {
        node.single_mut().text = format!("Vows judged: {}", vows.0);
    }
}
