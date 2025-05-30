
use bevy::{core_pipeline::{bloom::Bloom, prepass::{DepthPrepass, NormalPrepass}, tonemapping::{DebandDither, Tonemapping}}, math::FloatOrd, prelude::*, render::{camera::{ImageRenderTarget, RenderTarget}, render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages}, view::RenderLayers}, sprite::AlphaMode2d, window::WindowResized};

use crate::PostProcessSettings;

/// In-game resolution width.
pub const RES_WIDTH: u32 = 640;

/// In-game resolution height.
pub const RES_HEIGHT: u32 = 360;

/// Default render layers for pixel-perfect rendering.
/// You can skip adding this component, as this is the default.
pub const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);

/// Render layers for high-resolution rendering.
pub const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

#[derive(Component)]
pub struct CameraTarget;

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
    pub width: f32,
    pub height: f32,
    pub texel_size: f32,
    pub zoom: f32,
}

#[derive(Resource)]
pub struct ShowSettings{
    pub value: i32
}

pub struct PixelCamPlugin;

impl Plugin for PixelCamPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(WindowSize{
            width: 0.,
            height: 0.,
            texel_size: 0.,
            zoom: 5.
        })
        .insert_resource(ShowSettings{value: 0})
        .add_systems(Startup, setup_camera)
        .add_systems(Update, fit_canvas)
        .add_systems(Update, (update_settings, camera_follow, place_camera));
    }
}

fn update_settings(
    mut settings: Query<&mut PostProcessSettings>,
    keycode: Res<ButtonInput<KeyCode>>,
    mut show_depth: ResMut<ShowSettings>,
) {
    if keycode.just_pressed(KeyCode::Space) {
        for mut setting in settings.iter_mut() {
            show_depth.value = (show_depth.value + 1) % 3;
            setting.show_depth = (show_depth.value == 1) as u32;
            setting.show_normals = (show_depth.value == 2) as u32;
        }
    }
}

fn camera_follow(
    mut cam: Single<(&mut PixelCamera,&Camera), (With<PixelCamera>, Without<CameraPosition>)>,
    cam_t: Single<&GlobalTransform, With<CameraPosition>>,
    window: Res<WindowSize>,
    target_q: Single<&Transform, (With<CameraTarget>, Without<PixelCamera>, Without<CameraPosition>)>,
){
    let Ok(ray) = cam.1.world_to_viewport(&cam_t, target_q.translation) else {
        return;
    };
    if window.texel_size != 0. {
        cam.0.subpixel_position += Vec2::new((ray.x - RES_WIDTH as f32 / 2.)/RES_WIDTH as f32, -(ray.y - RES_HEIGHT as f32 / 2.)/RES_HEIGHT as f32) * 10.;
    }
}

fn camera_movement(
    mut window: ResMut<WindowSize>,
    mut cam: Single<(&mut PixelCamera, &mut Projection), With<PixelCamera>>,
    key_input: ResMut<ButtonInput<KeyCode>>,
    time: Res<Time>
){
    // let mut pixelcam = cam.single_mut();
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
    let mut zoom = 0.;
    if key_input.pressed(KeyCode::KeyQ) {
        zoom -= 1.0;
    }
    if key_input.pressed(KeyCode::KeyE) {
        zoom += 1.0;
    }
    if zoom != 0. {
        window.zoom += zoom * time.delta_secs() * 10.;
        window.zoom = window.zoom.clamp(1., 10.);
        
        *cam.1 = Projection::Orthographic(OrthographicProjection { scale: 1./window.zoom,
                far: 10000.,
                near: -1000., ..OrthographicProjection::default_3d() });

    }
    // cam.subpixel_position += dir.normalize_or_zero() * time.delta_secs() * 10.;
}

fn setup_camera(
    mut commands: Commands, 
    window: Res<WindowSize>,
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
    let mut bloom = Bloom::default();

    bloom.low_frequency_boost = 0.25;
        
    commands.spawn((
        Projection::from(OrthographicProjection{
            scale: 1./window.zoom,
            far: 10000.,
            near: -1000.,
            ..OrthographicProjection::default_3d()
        }),
        Camera {
            // render before the "main pass" camera
            // order: 1,
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            order: -1,
            target: RenderTarget::Image(ImageRenderTarget{handle: image_handle.clone(), scale_factor: FloatOrd(1.0)}),
            ..default()
        },
        Camera3d {
            depth_texture_usages: (TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING)
            .into(),
            ..default()
        },
        Transform::from_translation(Vec3::new(1., 1., -1.)).looking_at(Vec3::ZERO, Vec3::Y),
        PixelCamera{subpixel_position: Vec2::new(0.,0.)},
        PostProcessSettings {
            ..default()
        },
        Tonemapping::TonyMcMapface, 
        bloom,         
        DebandDither::Enabled,
        Msaa::Off,
        DepthPrepass,
        NormalPrepass,
        PIXEL_PERFECT_LAYERS,
    ));

    commands.spawn((CameraPosition, Transform::from_translation(Vec3::new(1., 1., -1.)).looking_at(Vec3::ZERO, Vec3::Y)));

    // spawn the canvas
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(RES_WIDTH as f32, RES_HEIGHT as f32))),
        MeshMaterial2d(materials.add(ColorMaterial{
            texture: Some(image_handle),
            alpha_mode: AlphaMode2d::Opaque,
            ..default()
        })),
        Bloom::NATURAL,
        Canvas,
        Msaa::Off,
        HIGH_RES_LAYERS,
    ));

    // the "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((
        Projection::from(OrthographicProjection{
            ..OrthographicProjection::default_2d()
        }),
        Camera2d::default(),
        OuterCamera,
        Msaa::Off,
        HIGH_RES_LAYERS
    ));
}

/// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut window: ResMut<WindowSize>,
    mut resize_events: EventReader<WindowResized>,
    mut projections: Single<&mut Projection, With<OuterCamera>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / (RES_WIDTH as f32 * 0.8).round();
        let v_scale = event.height / (RES_HEIGHT as f32 * 0.8).round();
        **projections = Projection::Orthographic(OrthographicProjection { scale: 1. / h_scale.min(v_scale).round(), ..OrthographicProjection::default_2d() });
        

        window.width = event.width;
        window.height = event.height;
        window.texel_size = h_scale.min(v_scale).round();
    }
}

#[derive(Component)]
pub struct CameraPosition;

fn place_camera(
    window: Res<WindowSize>,
    mut cam: Single<(&PixelCamera, &mut Transform, &mut Projection), (With<PixelCamera>, Without<CameraPosition>)>,
    mut canvas_q: Single<(&Canvas, &mut Transform), (With<Canvas>, Without<PixelCamera>, Without<CameraPosition>)>,
    mut cam_t: Single<&mut Transform, With<CameraPosition>>,
){
    let pos = cam.0.subpixel_position;
    let size = window.texel_size / window.zoom;
    if size == 0. {
        return;
    }

    let right = cam.1.right().mul_add(Vec3::ONE, Vec3::ZERO);
    let up = cam.1.up().mul_add(Vec3::ONE, Vec3::ZERO);

    let norm = Vec2::new(
        (pos.x / size).round() * size,
        (pos.y / size).round() * size,
    );
    let translate: Vec3 = right * norm.x + up * norm.y;

    cam.1.translation = translate;
    cam_t.translation = right * pos.x + up * pos.y;
    canvas_q.1.translation = Vec3::new(norm.x - pos.x, norm.y - pos.y, 0.) * window.zoom;
}