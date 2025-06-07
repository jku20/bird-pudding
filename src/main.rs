use std::{env, time::Duration};

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
    let mut app = App::new();

    // Add Bevy default plugins
    app.add_plugins(DefaultPlugins)
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
        .add_systems(Update, (keys_to_pause_time, heads_scared));

    if env::var("R").as_deref() == Ok("1") {
        app.add_plugins({
            let rec = revy::RecordingStreamBuilder::new("bevy-platformer")
                .save(format!("log-{}.rrd", chrono::offset::Local::now()))
                .unwrap();
            revy::RerunPlugin { rec }
        });
    }

    app.run();
}

#[derive(Component)]
struct ChainHead;

struct ChainDescription {
    tail_pos: Vec2,
    direction: Vec2,
    link_length: f32,
    link_width: f32,
    link_gap: f32,
    link_count: usize,
}

struct Chain {
    head_entity: Entity,
    tail_entity: Entity,
}

fn spawn_chain(commands: &mut Commands, description: ChainDescription) -> Chain {
    assert!(description.link_count > 0, "attempted to spawn empty chain");

    let direction = description.direction.normalize();

    // we act as if everything points to the right and handle rotation where needed

    let anchor_offset = (description.link_length + description.link_gap) / 2.0;
    let anchor_offset_first = Vec2::X * (description.link_length / 2.0);
    let anchor_offset_a = Vec2::X * anchor_offset;
    let anchor_offset_b = -anchor_offset_a;

    let tail_entity = commands
        .spawn((
            RigidBody::Static,
            Transform::from_xyz(description.tail_pos.x, description.tail_pos.y, 0.0),
        ))
        .id();

    let mut previous_entity = tail_entity;
    let mut current_pos = description.tail_pos + anchor_offset_first;

    for i in 0..description.link_count {
        let current_entity = commands
            .spawn((
                Sprite {
                    color: Color::srgb(0.0, 0.0, 1.0),
                    custom_size: Some(Vec2::new(description.link_length, description.link_width)),
                    ..default()
                },
                Transform::from_xyz(current_pos.x, current_pos.y, 0.0),
                Rotation::from_sin_cos(direction.y, direction.x),
                RigidBody::Dynamic,
                // LockedAxes::ROTATION_LOCKED,
                Collider::rectangle(description.link_length, description.link_width),
            ))
            .id();

        commands.spawn(
            RevoluteJoint::new(previous_entity, current_entity)
                .with_local_anchor_1(if i == 0 {
                    anchor_offset_first
                } else {
                    anchor_offset_a
                })
                .with_local_anchor_2(anchor_offset_b),
        );
        // if i > 0 {
        //     commands.spawn(
        //         DistanceJoint::new(previous_entity, current_entity)
        //             .with_limits(0.0, description.link_gap)
        //             .with_local_anchor_1(anchor_offset_a)
        //             .with_local_anchor_2(anchor_offset_b),
        //     );
        // }

        current_pos += direction * (description.link_length + description.link_gap);
        previous_entity = current_entity;
    }

    let head_entity = previous_entity;

    commands.entity(head_entity).insert(ChainHead);

    Chain {
        head_entity,
        tail_entity,
    }
}

#[derive(Component)]
struct Spike {
    direction: Vec2,
}

struct SpikeDescription {
    pos: Vec2,
    width: f32,
    height: f32,
    direction: Vec2,
}

fn spawn_spike(commands: &mut Commands, description: SpikeDescription) -> Entity {
    let direction = description.direction.normalize();

    commands
        .spawn((
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(description.width, description.height)),
                ..default()
            },
            Transform::from_xyz(description.pos.x, description.pos.y, 0.0),
            Rotation::from_sin_cos(direction.y, direction.x),
            RigidBody::Kinematic,
            Collider::rectangle(description.width, description.height),
            Spike {
                direction: description.direction,
            },
        ))
        .id()
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut time: ResMut<Time<Physics>>,
) {
    time.pause();

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

    // commands.spawn((
    //     Sprite {
    //         color: Color::srgb(0.0, 0.0, 1.0),
    //         custom_size: Some(Vec2::new(32.0, 32.0)),
    //         ..default()
    //     },
    //     // Transform::from_xyz(0.0, 100.0, 0.0),
    //     // GlobalTransform::default(),
    //     RigidBody::Dynamic,
    //     // LockedAxes::ROTATION_LOCKED,
    //     Collider::rectangle(32.0, 32.0),
    // ));

    // let a = commands
    //     .spawn((
    //         Sprite {
    //             color: Color::srgb(0.0, 0.0, 1.0),
    //             custom_size: Some(Vec2::new(8.0, 32.0)),
    //             ..default()
    //         },
    //         Transform::from_xyz(0.0, 100.0, 0.0),
    //         RigidBody::Dynamic,
    //         // LockedAxes::ROTATION_LOCKED,
    //         Collider::rectangle(8.0, 32.0),
    //     ))
    //     .id();
    // let b = commands
    //     .spawn((
    //         Sprite {
    //             color: Color::srgb(0.0, 0.0, 1.0),
    //             custom_size: Some(Vec2::new(8.0, 32.0)),
    //             ..default()
    //         },
    //         Transform::from_xyz(0.0, 142.0, 0.0),
    //         RigidBody::Dynamic,
    //         // LockedAxes::ROTATION_LOCKED,
    //         Collider::rectangle(8.0, 32.0),
    //     ))
    //     .id();
    // commands.spawn(
    //     RevoluteJoint::new(a, b)
    //         .with_local_anchor_1(Vec2::new(0.0, 16.0 + 2.0))
    //         .with_local_anchor_2(Vec2::new(0.0, -16.0 - 2.0)),
    // );

    // spawn_chain(
    //     &mut commands,
    //     ChainDescription {
    //         tail_pos: Vec2::new(0.0, 200.0),
    //         direction: Vec2::new(0.0, 1.0),
    //         link_length: 32.0,
    //         link_width: 8.0,
    //         link_gap: 4.0,
    //         link_count: 5,
    //     },
    // );
    spawn_chain(
        &mut commands,
        ChainDescription {
            tail_pos: Vec2::new(240.0, 120.0),
            direction: Vec2::new(0.5, -0.8),
            link_length: 32.0,
            link_width: 8.0,
            link_gap: 4.0,
            link_count: 5,
        },
    );
    spawn_chain(
        &mut commands,
        ChainDescription {
            tail_pos: Vec2::new(-100.0, 220.0),
            direction: Vec2::new(-1.0, -0.2),
            link_length: 32.0,
            link_width: 8.0,
            link_gap: 4.0,
            link_count: 5,
        },
    );
    spawn_spike(
        &mut commands,
        SpikeDescription {
            pos: Vec2::new(200.0, 0.0),
            width: 20.0,
            height: 20.0,
            direction: Vec2::new(1.0, 0.0),
        },
    );
}

fn keys_to_pause_time(mut time: ResMut<Time<Physics>>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }

    if keys.just_pressed(KeyCode::Enter) && time.is_paused() {
        if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            time.advance_by(Duration::from_millis(10));
        } else {
            time.advance_by(Duration::from_millis(100));
        }
    }
}

fn heads_scared(
    mut commands: Commands,
    mut heads: ParamSet<(
        Query<(&Transform, Entity), (With<ChainHead>, Without<Spike>)>,
        Query<(&Transform, &mut ExternalForce), (With<ChainHead>, Without<Spike>)>,
    )>,
    spikes: Query<(&Spike, &Transform), (With<Spike>, Without<ChainHead>)>,
) {
    for (_, head_entity) in heads.p0().iter() {
        commands
            .entity(head_entity)
            .insert(ExternalForce::new(Vec2::ZERO).with_persistence(true));
    }
    for (spike, spike_transform) in spikes {
        for (head_transform, mut head_force) in heads.p1().iter_mut() {
            head_force.apply_force(spike.direction * 100000.0);
            println!("spike force {head_force:?}");
        }
    }
}
