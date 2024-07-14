use bevy::prelude::*;

pub fn draw_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle = asset_server.load("images/crosshair.png");

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
                image: UiImage::new(texture_handle),
                ..Default::default()
            });
        });
}
