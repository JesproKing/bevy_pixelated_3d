
use bevy::{core_pipeline::prepass::{DepthPrepass, NormalPrepass}, prelude::*, render::{camera::RenderTarget, render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages}, view::RenderLayers}, sprite::AlphaMode2d, window::WindowResized};

use crate::PostProcessSettings;

/// In-game resolution width.
pub const RES_WIDTH: u32 = 320;

/// In-game resolution height.
pub const RES_HEIGHT: u32 = 180;

/// Default render layers for pixel-perfect rendering.
/// You can skip adding this component, as this is the default.
pub const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);

/// Render layers for high-resolution rendering.
pub const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component)]
pub struct PixelCamera{
    pub subpixel_position: Vec2
}

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Component)]
struct OuterCamera;

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Resource)]
pub struct WindowSize{
    width: f32,
    height: f32,
    pub texel_size: f32,
}

pub struct PixelCamPlugin;

impl Plugin for PixelCamPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(WindowSize{
            width: 0.,
            height: 0.,
            texel_size: 0.,
        })
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (place_camera, fit_canvas.run_if(on_event::<WindowResized>)));
    }
}

fn camera_movement(
    mut cam: Query<&mut PixelCamera, With<PixelCamera>>,
    key_input: ResMut<ButtonInput<KeyCode>>,
    time: Res<Time>
){
    let mut pixelcam = cam.single_mut();
    let mut dir = Vec2::splat(0.0);
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

    pixelcam.subpixel_position += dir.normalize_or_zero() * time.delta_secs() * 50.;
}

fn setup_camera(
    mut commands: Commands, 
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // this Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    // this camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas
    commands.spawn((
        Projection::from(OrthographicProjection::default_2d()),
        Camera {
            // render before the "main pass" camera
            order: -1,
            target: RenderTarget::Image(image_handle.clone()),
            ..default()
        },
        Camera3d {
            depth_texture_usages: (TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING)
            .into(),
            ..default()
        },
        Transform::from_translation(Vec3::new(0., 5., -5.)).looking_at(Vec3::ZERO, Vec3::Y),
        PixelCamera{subpixel_position: Vec2::new(0.,5.)},
        PostProcessSettings {
            ..default()
        },
        Msaa::Off,
        DepthPrepass,
        NormalPrepass,
        PIXEL_PERFECT_LAYERS,
    ));

    // spawn the canvas
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(RES_WIDTH as f32, RES_HEIGHT as f32))),
        MeshMaterial2d(materials.add(ColorMaterial{
            texture: Some(image_handle),
            alpha_mode: AlphaMode2d::Opaque,
            ..default()
        })),
        Canvas,
        HIGH_RES_LAYERS,
    ));

    // the "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((
        Camera2d::default(),
        OuterCamera,
        HIGH_RES_LAYERS
    ));
}

/// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut window: ResMut<WindowSize>,
    mut resize_events: EventReader<WindowResized>,
    mut projections: Query<&mut OrthographicProjection, With<OuterCamera>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / (RES_WIDTH as f32 * 0.8).round();
        let v_scale = event.height / (RES_HEIGHT as f32 * 0.8).round();
        let mut projection = projections.single_mut();
        projection.scale = 1. / h_scale.min(v_scale).round();

        window.width = event.width;
        window.height = event.height;
        window.texel_size = v_scale.round();
    }
}

fn place_camera(
    window: Res<WindowSize>,
    mut cam: Query<(&PixelCamera, &mut Transform), With<PixelCamera>>,
    mut canvas_q: Query<(&Canvas, &mut Transform), (With<Canvas>, Without<PixelCamera>)>,
){
    let (pixelcam, mut t) = cam.single_mut();
    let (_, mut t_c) = canvas_q.single_mut();
    let pos = pixelcam.subpixel_position;
    let size = window.texel_size;

    let right = t.right().mul_add(Vec3::ONE, Vec3::ZERO);
    let up = t.up().mul_add(Vec3::ONE, Vec3::ZERO);

    let norm = Vec2::new(
        (pos.x / size).round() * size,
        (pos.y / size).round() * size,
    );
    let translate: Vec3 = right * norm.x + up * norm.y + Vec3::Y * 5.;

    t.translation = translate;
    t_c.translation = Vec3::new(norm.x - pos.x, norm.y - pos.y, 0.);
}