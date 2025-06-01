use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

fn main() {
    App::new()
        // Add Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Add bevy_ecs_tiled plugin: note that bevy_ecs_tilemap::TilemapPlugin
        // will be automatically added as well if it's not already done
        .add_plugins(TiledMapPlugin::default())
        // Add our startup function to the schedule and run the app
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a Bevy 2D camera
    commands.spawn(Camera2d);

    // Load a map asset and retrieve the corresponding handle
    let map_handle: Handle<TiledMap> = asset_server.load("test_level.tmx");

    // Spawn a new entity with this handle
    commands.spawn((TiledMapHandle(map_handle), TilemapAnchor::Center));
}
