use bevy::prelude::*;
use bevy_gltf_collider::get_scene_colliders;
use bevy_rapier3d::prelude::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, States, Default)]
enum GameState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Default, Resource)]
struct GameAssets {
    monkey_scene: Handle<Scene>,
    monkey_colliders: Vec<(Collider, Transform)>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::rgb(0.7, 0.9, 1.0)))
        .add_plugins((RapierPhysicsPlugin::<NoUserData>::default(), RapierDebugRenderPlugin::default()))
        .insert_resource(AmbientLight {
            brightness: 0.5,
            ..default()
        })
        .add_systems(OnEnter(GameState::Loading), (start_assets_loading,))
        .add_systems(Update, (check_if_loaded,).run_if(in_state(GameState::Loading)))
        .add_systems(OnEnter(GameState::Loaded), (spawn_monkeys,))
        .run();
}

// load the monkey scene from the gltf file
fn start_assets_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        monkey_scene: asset_server.load("models/monkey.glb#Scene0"),
        ..default()
    });
}

// check if the monkey scene is loaded and if so, get the colliders from it
fn check_if_loaded(
    mut scenes: ResMut<Assets<Scene>>,
    mut game_assets: ResMut<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let scene = if let Some(scene) = scenes.get_mut(&game_assets.monkey_scene) {
        scene
    } else {
        return;
    };

    // get_scene_colliders should be called only once per scene as it will remove the colliders meshes from it
    game_assets.monkey_colliders = get_scene_colliders(&mut meshes, &mut scene.world)
        .expect("Failed to create monkey colliders");

    game_state.set(GameState::Loaded);
}

// spawn objects
fn spawn_monkeys(mut commands: Commands, game_assets: Res<GameAssets>) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-5.0, 10.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    let base_pos = Vec3::new(0.0, 4.0, 0.0);
    let spread = 10.0;
    for i in 0..10 {
        let x = rand::random();
        let y = rand::random();
        let z = rand::random();

        let pos = base_pos + (Vec3::new(x, y, z) * spread - Vec3::ONE * spread / 2.0);

        // Spawn monkeys
        commands
            .spawn((
                RigidBody::Dynamic,
                Restitution::coefficient(0.7),
                Name::new(format!("monkey_{}", i)),
                SceneBundle {
                    scene: game_assets.monkey_scene.clone(),
                    transform: Transform::from_translation(pos),
                    ..default()
                },
            ))
            // Spawn colliders
            .with_children(|parent| {
                for (collider, transform) in game_assets.monkey_colliders.iter() {
                    parent.spawn((
                        collider.clone(),
                        TransformBundle::from_transform(*transform),
                    ));
                }
            });
    }
}
