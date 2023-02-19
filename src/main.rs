use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::ButtonState,
    math::Vec3Swizzles,
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::AsBindGroup,
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};

mod camera;
mod simulator;

use camera::{CameraPlugin, WorldClickEvent};
use simulator::*;

#[derive(Component)]
struct CircuitComponent {
    simulator: Option<Simulator>,
}

impl Default for CircuitComponent {
    fn default() -> Self {
        Self { simulator: None }
    }
}

#[derive(Bundle)]
struct CircuitBundle<M: Material2d> {
    circuit: CircuitComponent,

    #[bundle]
    material_mesh: MaterialMesh2dBundle<M>,
}

impl<M: Material2d> Default for CircuitBundle<M> {
    fn default() -> Self {
        Self {
            circuit: default(),
            material_mesh: default(),
        }
    }
}

#[derive(AsBindGroup, Clone, TypeUuid)]
#[uuid = "2ef05c0a-d55f-4069-9a65-f7ccc072f3e4"]
struct CircuitMaterial {
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    overlay_texture: Option<Handle<Image>>,
}

impl Material2d for CircuitMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/circuit_material.vert".into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/circuit_material.frag".into()
    }

    // Bevy assumes by default that vertex shaders use the "vertex" entry point
    // and fragment shaders use the "fragment" entry point (for WGSL shaders).
    // GLSL uses "main" as the entry point, so we must override the defaults here
    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();
        Ok(())
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(CameraPlugin)
        .add_plugin(Material2dPlugin::<CircuitMaterial>::default())
        .add_startup_system(setup)
        .add_system(circuit_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut circuit_materials: ResMut<Assets<CircuitMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle: Handle<Image> = asset_server.load("8bit_cpu.png");

    commands.spawn(CircuitBundle {
        material_mesh: MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad::new(Vec2::new(1024.0, 1024.0))))
                .into(),
            material: circuit_materials.add(CircuitMaterial {
                texture: texture_handle.clone(),
                overlay_texture: None,
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        ..default()
    });
}

fn circuit_system(
    mut ev_world_click: EventReader<WorldClickEvent>,
    meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut circuit_materials: ResMut<Assets<CircuitMaterial>>,
    mut circuit_query: Query<(
        &mut CircuitComponent,
        &Handle<CircuitMaterial>,
        &Transform,
        &Mesh2dHandle,
    )>,
) {
    circuit_query.for_each_mut(|(mut circuit, handle, transform, mesh_handle)| {
        let material = if let Some(material) = circuit_materials.get_mut(handle) {
            material
        } else {
            return;
        };

        let image = if let Some(image) = images.get(&material.texture) {
            image
        } else {
            return;
        };

        let mesh = if let Some(mesh) = meshes.get(&mesh_handle.0) {
            mesh
        } else {
            return;
        };

        let simulator = circuit
            .simulator
            .get_or_insert_with(|| Simulator::from_image(image));

        let overlay_image_handle = material.overlay_texture.get_or_insert_with(|| {
            let image = &simulator.raw_image;

            let overlay_image = Image::new(
                image.texture_descriptor.size,
                image.texture_descriptor.dimension,
                vec![255u8; image.data.len()],
                image.texture_descriptor.format,
            );

            images.add(overlay_image)
        });

        let overlay_image = if let Some(overlay_image) = images.get_mut(&overlay_image_handle) {
            overlay_image
        } else {
            return;
        };

        for event in ev_world_click.iter() {
            let world_pos = &event.pos;

            let aabb = if let Some(aabb) = mesh.compute_aabb() {
                aabb
            } else {
                continue;
            };

            let rect =
                Rect::from_center_half_size(transform.translation.xy(), aabb.half_extents.xy());

            let does_collide = rect.contains(*world_pos);

            if !does_collide {
                continue;
            }

            let relative_pos = *world_pos - (rect.center() - rect.half_size());

            let x = relative_pos.x as u32;
            let y = (simulator.height as f32 - relative_pos.y) as u32;

            simulator.set(x, y, event.state == ButtonState::Pressed);
        }

        simulator.simulate(20);

        let components = overlay_image
            .texture_descriptor
            .format
            .describe()
            .components as usize;
        let mut pixels: Vec<&mut [u8]> = overlay_image.data.chunks_exact_mut(components).collect();
        let mut rows: Vec<&mut [&mut [u8]]> =
            pixels.chunks_exact_mut(simulator.width as usize).collect();

        for y in 0..simulator.height as usize {
            for x in 0..simulator.width as usize {
                let wire_id = simulator.wire_map[y][x];

                if wire_id == -1 {
                    continue;
                }

                let wire_state = simulator.wires[wire_id as usize].state;

                let pixel = &mut rows[y][x];

                if wire_state {
                    pixel[..components - 1]
                        .iter_mut()
                        .for_each(|value| *value = 255);
                } else {
                    pixel[..components - 1]
                        .iter_mut()
                        .for_each(|value| *value = 80);
                }
            }
        }
    });
}
