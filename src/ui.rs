use bevy::prelude::*;

#[derive(Component, Default)]
pub struct SelectedPosition {}

pub fn draw_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let crosshair_handle = asset_server.load("images/crosshair.png");
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                margin: UiRect::all(Val::Auto),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage::new(crosshair_handle),
                ..Default::default()
            });
        });

    let toolbar_handle = asset_server.load("images/toolbar.png");
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(392.0),
                height: Val::Px(88.0),
                align_self: AlignSelf::FlexEnd,
                justify_self: JustifySelf::Center,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::NONE),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage::new(toolbar_handle),
                ..Default::default()
            });
        });

    let selected_handle = asset_server.load("images/selected.png");
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(72.0),
                    height: Val::Px(72.0),
                    align_self: AlignSelf::FlexEnd,
                    justify_self: JustifySelf::Center,
                    margin: UiRect {
                        bottom: Val::Px(8.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..Default::default()
            },
            SelectedPosition::default(),
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage::new(selected_handle),
                ..Default::default()
            });
        });
}
