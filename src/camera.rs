use bevy::{app::Events, input::mouse::MouseMotion, prelude::*};

// const CAMERA_MOVE_SPEED: f32 = 200.;
const CAMERA_MOVE_SPEED: f32 = 100.;
const CAMERA_SENSITIVITY: f32 = 0.00012;

pub struct CameraPlugin;

#[derive(Default)]
struct MouseState {
    pitch: f32,
    yaw: f32,
}

#[derive(Component)]
pub struct SceneCam;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseState::default())
            .add_startup_system(setup_cursor_lock.system())
            .add_startup_system(setup_camera.system())
            .add_system(camera_movement.system())
            .add_system(exit_control.system())
            .add_system(camera_mouse_movement.system());
    }
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            perspective_projection: PerspectiveProjection {
                // Adjust render distance to prevent planets disappear
                far: 10000.0,
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., 0.).looking_at(Vec3::ZERO, Vec3::X),
            ..Default::default()
        })
        .insert(SceneCam);
}

// grab & lock cursor when game first starts
fn setup_cursor_lock(mut windows: ResMut<Windows>) {
    let mut window = windows.get_primary_mut().unwrap();
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

fn camera_movement(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    mut query: Query<&mut Transform, With<SceneCam>>,
) {
    let mut cam_tf = query.single_mut();
    let window = windows.get_primary().unwrap();
    let mut v = Vec3::ZERO;
    let local_z = cam_tf.local_z();
    let forward = -Vec3::new(local_z.x, 0., local_z.z);
    let right = Vec3::new(local_z.z, 0., -local_z.x);

    for key in input.get_pressed() {
        if window.cursor_locked() {
            match key {
                KeyCode::W => v += forward,
                KeyCode::S => v -= forward,
                KeyCode::A => v -= right,
                KeyCode::D => v += right,
                KeyCode::E => v += Vec3::Y,
                KeyCode::Q => v -= Vec3::Y,
                _ => (),
            }
        }
    }

    v = v.normalize_or_zero();
    cam_tf.translation += v * time.delta_seconds() * CAMERA_MOVE_SPEED;
}

fn exit_control(input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

// Handles looking around if cursor is locked
fn camera_mouse_movement(
    windows: Res<Windows>,
    mut motion_evr: EventReader<MouseMotion>,
    mut state: ResMut<MouseState>,
    mut query: Query<&mut Transform, With<SceneCam>>,
) {
    let window = windows.get_primary().unwrap();
    let mut camera_tf = query.single_mut();

    for ev in motion_evr.iter() {
        if window.cursor_locked() && window.is_focused() {
            // Using smallest of height or width ensures equal vertical and horizontal sensitivity
            let window_scale = window.height().min(window.width());

            state.pitch -= (CAMERA_SENSITIVITY * ev.delta.y * window_scale).to_radians();
            state.yaw -= (CAMERA_SENSITIVITY * ev.delta.x * window_scale).to_radians();
            state.pitch = state.pitch.clamp(-1.54, 1.54);

            // Order is important to prevent unintended roll
            camera_tf.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw)
                * Quat::from_axis_angle(Vec3::X, state.pitch);
        }
    }
}
