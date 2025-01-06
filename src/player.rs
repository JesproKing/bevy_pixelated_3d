
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
    mut player: Query<(&mut Transform, &mut Player), (With<Player>, Without<PixelCamera>)>,
    mut cam: Query<(&Camera, &Transform, &mut PixelCamera), With<PixelCamera>>,
    key_input: ResMut<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window: Res<WindowSize>,
){

    let size = window.texel_size;
    if size == 0. {
        return;
    }
    let (_camera, cam_t, mut pixelcam) = cam.single_mut();
    let right = cam_t.right().mul_add(Vec3::ONE, Vec3::ZERO);
    let up = cam_t.up().mul_add(Vec3::ONE, Vec3::ZERO);

    let (mut p_t, mut p) = player.single_mut();
    let mut dir = Vec3::splat(0.0);
    if key_input.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if key_input.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    if key_input.pressed(KeyCode::KeyW) {
        dir.z += 1.0;
    }
    if key_input.pressed(KeyCode::KeyS) {
        dir.z -= 1.0;
    }

    let mut pos = Vec3::new(p.x, 13., p.y) + dir.normalize_or_zero() * time.delta_secs() * 50.;
    p.x = pos.x;
    p.y = pos.z;
    let norm = Vec2::new(
        ((pos/right).x / (size / 3.)).round() * (size / 3.),
        ((pos/up).z / (size / 3.)).round() * (size / 3.),
    );
    pos += Vec3::new(0.,0.,-5.);
    p_t.translation = Vec3::new(norm.x,13.,norm.y);
    pixelcam.subpixel_position = Vec2::new((-pos/right).x, (pos/up).z);
}
