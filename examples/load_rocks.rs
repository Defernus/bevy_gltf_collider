use bevy::prelude::*;
use bevy_gltf_collider::get_scene_colliders;
use bevy_rapier3d::prelude::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum GameState {
    Loading,
    Loaded,
}

#[derive(Default, Resource)]
struct GameAssets {
    rock_scene: Handle<Scene>,
    rock_colliders: Vec<(Collider, Transform)>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(GameState::Loading)
        .insert_resource(ClearColor(Color::rgb(0.7, 0.9, 1.0)))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(AmbientLight {
            brightness: 0.5,
            ..default()
        })
        .add_system_set(SystemSet::on_enter(GameState::Loading).with_system(start_assets_loading))
        .add_system_set(SystemSet::on_update(GameState::Loading).with_system(check_if_loaded))
        .add_system_set(SystemSet::on_enter(GameState::Loaded).with_system(spawn_rocks))
        .run();
}

// load the rock scene from the gltf file
fn start_assets_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        rock_scene: asset_server.load("models/rock.gltf#Scene0"),
        ..default()
    });
}

// check if the rock scene is loaded and if so, get the colliders from it
fn check_if_loaded(
    mut scenes: ResMut<Assets<Scene>>,
    mut game_assets: ResMut<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut game_state: ResMut<State<GameState>>,
) {
    let scene = if let Some(scene) = scenes.get_mut(&game_assets.rock_scene) {
        scene
    } else {
        return;
    };

    // get_scene_colliders should be called only once per scene as it will remove the colliders meshes from it
    game_assets.rock_colliders = get_scene_colliders(&mut meshes, &mut scene.world)
        .expect("Failed to create rock colliders");

    game_state.set(GameState::Loaded).unwrap();
}

// spawn objects
fn spawn_rocks(mut commands: Commands, game_assets: Res<GameAssets>) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-1.0, 2.0, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Add directional light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(4.0, 4.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Create the ground.
    commands.spawn((
        Collider::cuboid(100.0, 1.0, 100.0),
        TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)),
    ));

    let base_pos = Vec3::new(0.0, 2.0, 0.0);
    let spread = 1.0;
    for i in 0..100 {
        let x = rand::random();
        let y = rand::random();
        let z = rand::random();

        let pos = base_pos + (Vec3::new(x, y, z) * spread - Vec3::ONE * spread / 2.0);

        // Spawn rocks
        commands
            .spawn((
                RigidBody::Dynamic,
                Restitution::coefficient(0.7),
                Name::new(format!("rock_{}", i)),
                SceneBundle {
                    scene: game_assets.rock_scene.clone(),
                    transform: Transform::from_translation(pos),
                    ..default()
                },
            ))
            // Spawn colliders
            .with_children(|parent| {
                for (collider, transform) in game_assets.rock_colliders.iter() {
                    parent.spawn((
                        collider.clone(),
                        TransformBundle::from_transform(transform.clone()),
                    ));
                }
            });
    }
}
