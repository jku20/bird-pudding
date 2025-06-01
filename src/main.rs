use avian2d::{math::Vector, prelude::*};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

// https://github.com/adrien-bon/bevy_ecs_tiled/blob/ee458ad464e8ea7cea22c7923efb911945b5d710/examples/physics_avian_controller.rs#L96C1-L117C2
#[derive(Default, Debug, Clone, Reflect)]
#[reflect(Default, Debug)]
struct MyCustomAvianPhysicsBackend(TiledPhysicsAvianBackend);

impl TiledPhysicsBackend for MyCustomAvianPhysicsBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        tiled_map: &TiledMap,
        filter: &TiledNameFilter,
        collider: &TiledCollider,
        anchor: &TilemapAnchor,
    ) -> Vec<TiledColliderSpawnInfos> {
        let colliders = self
            .0
            .spawn_colliders(commands, tiled_map, filter, collider, anchor);
        for c in &colliders {
            commands.entity(c.entity).insert(RigidBody::Static);
        }
        colliders
    }
}

fn main() {
    App::new()
        // Add Bevy default plugins
        .add_plugins(DefaultPlugins)
        // Add bevy_ecs_tiled plugin: note that bevy_ecs_tilemap::TilemapPlugin
        // will be automatically added as well if it's not already done
        .add_plugins(TiledMapPlugin::default())
        .add_plugins(TiledPhysicsPlugin::<MyCustomAvianPhysicsBackend>::default())
        // Load Avian main plugin
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .add_plugins((
            PhysicsDebugPlugin::default(),
            PhysicsDiagnosticsPlugin,
            PhysicsDiagnosticsUiPlugin,
        ))
        .insert_resource(Gravity(Vector::NEG_Y * 100.0))
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
    commands.spawn((
        TiledMapHandle(map_handle),
        TilemapAnchor::Center,
        TiledPhysicsSettings::<TiledPhysicsAvianBackend>::default(),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 0.0, 1.0),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        // Transform::from_xyz(0.0, 100.0, 0.0),
        // GlobalTransform::default(),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        // Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        Collider::rectangle(32.0, 32.0),
    ));
}
