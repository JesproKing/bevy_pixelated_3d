
use bevy::prelude::*;
use crate::pixel_cam::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player{
    pub x: f32,
    pub y: f32,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, player_movement);
    }
}

fn player_movement(
    mut player: Single<(&mut Transform, &mut Player), (With<Player>, Without<PixelCamera>)>,
    mut cam: Single<(&Camera, &Transform, &mut PixelCamera), With<PixelCamera>>,
    key_input: ResMut<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window: Res<WindowSize>,
){
    let right = cam.1.right().mul_add(Vec3::ONE, Vec3::ZERO);
    let forward = cam.1.forward().mul_add(Vec3::ONE, Vec3::ZERO).with_y(0.);

    let mut dir = Vec3::splat(0.0);
    if key_input.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if key_input.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    if key_input.pressed(KeyCode::KeyW) {
        dir.y += 1.0;
    }
    if key_input.pressed(KeyCode::KeyS) {
        dir.y -= 1.0;
    }

    dir = dir.normalize_or_zero();
    let mut pos = dir.x * right * time.delta_secs() * 50. + dir.y * forward * time.delta_secs() * 50.;
    
    player.0.translation += pos;
}
