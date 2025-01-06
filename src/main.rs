//! Shows how to create graphics that snap to the pixel grid by rendering to a texture in 2D

use bevy::prelude::*;
use bevy_pixelated_3d::*;
use std::f32::consts::PI;


#[derive(Resource)]
pub struct ShowSettings{
    value: i32
}

fn main() {
    App::new()
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
    })
    .insert_resource(ShowSettings{value: 0})
        .add_plugins(PixelCamPlugin)
        .add_plugins(PostProcessPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, setup_mesh)
        .add_systems(Update, (rotate_rotatable, rotate, update_settings))
        .run();
}

#[derive(Component)]
struct Rotate{
    timer: Timer
}
#[derive(Component)]
struct Rotatable{
    rotation: f32
}


fn rotate_rotatable(
    mut commands: Commands,
    mut rot: Query<Entity, (With<Rotatable>, Without<Rotate>)>,
    key_input: ResMut<ButtonInput<KeyCode>>,
){
    for e in &mut rot {
        if key_input.just_pressed(KeyCode::KeyR) {
            commands.entity(e).insert(Rotate{
                timer: Timer::from_seconds(1., TimerMode::Once)
            });
        }
    }
}

/// Spawns a capsule mesh on the pixel-perfect layer.
fn setup_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(1.,0.,1.)))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.5,0.5,0.5))),
        Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(90.)),
        Rotatable{
            rotation: 0.
        },
        PIXEL_PERFECT_LAYERS,
    )).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::new(1.05,0.15,0.05))))),
            MeshMaterial3d(materials.add(Color::linear_rgb(0.5,0.5,0.5))),
            Transform::from_xyz(0., 0.075,0.5),
            PIXEL_PERFECT_LAYERS,
        ));
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::new(1.05,0.15,0.05))))),
            MeshMaterial3d(materials.add(Color::linear_rgb(0.5,0.5,0.5))),
            Transform::from_xyz(0., 0.075,-0.5),
            PIXEL_PERFECT_LAYERS,
        ));
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::new(0.05,0.15,1.05))))),
            MeshMaterial3d(materials.add(Color::linear_rgb(0.5,0.5,0.5))),
            Transform::from_xyz(0.5, 0.075,0.),
            PIXEL_PERFECT_LAYERS,
        ));
        parent.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::new(0.05,0.15,1.05))))),
            MeshMaterial3d(materials.add(Color::linear_rgb(0.5,0.5,0.5))),
            Transform::from_xyz(-0.5, 0.075,0.),
            PIXEL_PERFECT_LAYERS,
        ));
    });

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5,1.0))),
        MeshMaterial3d(materials.add(Color::linear_rgb(0.1,0.5,0.1))),
        Transform::from_xyz(0., 8.,0.).with_scale(Vec3::splat(12.)),
            Player{
                x: 0.,
                y: 0.
            },
            PIXEL_PERFECT_LAYERS,
        ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 4000.,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            3. * PI / 4.,
            -PI / 4.,
        )),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(25.0, 2.0, 25.0)),
        PointLight{
            intensity: 20_000_000.0,
            radius: 30.,
            color: Color::linear_rgb(0.1, 0.5, 0.1),
            ..default()
        },
    ));
}

/// Rotates entities to demonstrate grid snapping.
fn rotate(
    mut commands: Commands,
    time: Res<Time>, 
    mut transforms: Query<(Entity, &mut Transform, &mut Rotate), With<Rotate>>,
    mut rotatables: Query<&mut Rotatable>
) {
    for (e, mut transform, mut rotate) in &mut transforms {
        let mut y = 0.;
        if let Ok(rot) = rotatables.get(e)  {
            y = rot.rotation;
        }
        rotate.timer.tick(time.delta());
        let t = (rotate.timer.fraction() * 36.).round() / 36.;
        transform.rotation = Quat::from_rotation_y(y*PI*0.5 + PI*0.5*t);
        if rotate.timer.finished(){
            if let Ok(mut rot) = rotatables.get_mut(e)  {
                y = (y + 1.) % 4.;
                rot.rotation = y;
            }
            transform.rotation = Quat::from_rotation_y(y*PI*0.5);
            if rotate.timer.mode() == TimerMode::Once {
                commands.entity(e).remove::<Rotate>();
            }
        }
    }
}


// Change the intensity over time to show that the effect is controlled from the main world
fn update_settings(
    mut settings: Query<&mut PostProcessSettings>,
    keycode: Res<ButtonInput<KeyCode>>,
    mut show_depth: ResMut<ShowSettings>,
) {
    if keycode.just_pressed(KeyCode::Space) {
        for mut setting in &mut settings {
            show_depth.value = (show_depth.value + 1) % 3;
            setting.show_depth = (show_depth.value == 1) as u32;
            setting.show_normals = (show_depth.value == 2) as u32;
        }
    }
}