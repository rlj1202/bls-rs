use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseWheel},
        ButtonState,
    },
    math::Vec3Swizzles,
    prelude::*,
};

#[derive(Resource)]
struct MouseSystem {
    prev_cursor_pos: Option<Vec2>,
}

pub struct WorldClickEvent {
    pub pos: Vec2,
    pub state: ButtonState,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MouseSystem {
            prev_cursor_pos: None,
        })
        .add_event::<WorldClickEvent>()
        .add_startup_system(setup)
        .add_system(mouse_click_system);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            near: -10.0,
            ..default()
        },
        ..default()
    });
}

fn mouse_click_system(
    mut camera_entity_query: Query<(&mut Transform, &GlobalTransform, &Camera)>,
    mut mouse_system: ResMut<MouseSystem>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut ev_world_click: EventWriter<WorldClickEvent>,
) {
    let camera_entity = camera_entity_query.single_mut();
    let mut camera_transform = camera_entity.0;
    let camera_global_trans = camera_entity.1;
    let camera = camera_entity.2;

    for event in mouse_button_input_events.iter() {
        match (event.button, event.state, mouse_system.prev_cursor_pos) {
            (MouseButton::Left, state, Some(pos)) => {
                let world_pos = camera.viewport_to_world(camera_global_trans, pos);

                if let Some(world_pos) = world_pos {
                    let world_pos = world_pos.origin;

                    ev_world_click.send(WorldClickEvent {
                        pos: world_pos.xy(),
                        state,
                    });
                }
            }
            _ => (),
        }
    }

    for event in cursor_moved_events.iter() {
        let cur_cursor_pos = event.position;

        match (
            mouse_button_input.pressed(MouseButton::Middle),
            mouse_system.prev_cursor_pos,
        ) {
            (true, Some(prev_cursor_pos)) => {
                let delta = cur_cursor_pos - prev_cursor_pos;
                let delta = Vec3::new(delta.x, delta.y, 0.0);
                let delta = delta * camera_transform.scale;

                camera_transform.translation -= delta;
            }
            _ => (),
        }

        mouse_system.prev_cursor_pos = Some(cur_cursor_pos);
    }

    for event in mouse_wheel_events.iter() {
        if event.y > 0.0 {
            camera_transform.scale /= Vec3::new(1.1, 1.1, 1.0);
        } else {
            camera_transform.scale *= Vec3::new(1.1, 1.1, 1.0);
        }

        camera_transform.scale = camera_transform
            .scale
            .max(Vec3::new(0.1, 0.1, 1.0))
            .min(Vec3::new(100.0, 100.0, 1.0));
    }
}
