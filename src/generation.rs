use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

pub struct GenerationPlugin;

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init)
            .insert_resource(RegenerateTimer(Timer::from_seconds(0.25, TimerMode::Once)))
            .add_systems(Update, trigger_generate)
            .add_systems(Update, generation);
    }
}

#[derive(Component)]
struct ResultContainer;

#[derive(Resource)]
struct GenerateAssets {
    mesh: Handle<Mesh>,
}

#[derive(Resource)]
struct RegenerateTimer(Timer);

#[derive(Resource)]
struct GenerateCommand;

fn init(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(shape::Quad::new(CELL_SIZE * Vec2::ONE).into());

    commands.insert_resource(GenerateAssets { mesh });
}

fn trigger_generate(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<RegenerateTimer>,
    window: Query<&Window, Changed<Window>>,
    mut last_size: Local<UVec2>,
) {
    let window_res = &window.single().resolution;
    let new_size: UVec2 = [window_res.physical_width(), window_res.physical_height()].into();
    if new_size != *last_size {
        *last_size = new_size;
        timer.0.reset();
    }

    if timer.0.tick(time.delta()).just_finished() {
        commands.insert_resource(GenerateCommand);
    }
}

fn generation(
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
    assets: Res<GenerateAssets>,
    previous_results: Query<Entity, With<ResultContainer>>,
    generate: Option<Res<GenerateCommand>>,
    mut commands: Commands,
) {
    if let Some(generate) = generate {
        if !generate.is_added() {
            return;
        }

        previous_results
            .iter()
            .for_each(|entity| commands.entity(entity).despawn_recursive());

        let width = window.single().resolution.width();
        let height = window.single().resolution.height();

        let cells_x = (width / CELL_SIZE).ceil() as i32;
        let cells_y = (height / CELL_SIZE).ceil() as i32;

        let x_offset_correction = 0.5; //if cells_x % 2 == 0 { 0.5 } else { 0.0 };
        let y_offset_correction = 0.5; //if cells_y % 2 == 0 { 0.5 } else { 0.0 };

        let x_offset = CELL_SIZE * (-0.5 * cells_x as f32 + x_offset_correction);
        let y_offset = CELL_SIZE * (-0.5 * cells_y as f32 + y_offset_correction);

        info!("generating {}x{}", cells_x, cells_y);

        let mut parent = commands.spawn((ResultContainer, SpatialBundle::default()));

        parent.with_children(|cb| {
            (0..cells_x).for_each(|x| {
                (0..cells_y).for_each(|y| {
                    let color_index = (x + cells_x * y) as usize % COLORS.len();
                    cb.spawn(MaterialMesh2dBundle {
                        mesh: assets.mesh.clone().into(),
                        material: materials.add(ColorMaterial::from(COLORS[color_index])),
                        transform: Transform::from_translation(Vec3::new(
                            x_offset + CELL_SIZE * x as f32,
                            y_offset + CELL_SIZE * y as f32,
                            0.,
                        )),
                        ..default()
                    });
                });
            });
        });

        commands.remove_resource::<GenerateCommand>();
    }
}

const CELL_SIZE: f32 = 128.0f32;

const COLORS: [bevy::prelude::Color; 6] = [
    Color::ALICE_BLUE,
    Color::ORANGE_RED,
    Color::TURQUOISE,
    Color::DARK_GREEN,
    Color::VIOLET,
    Color::BISQUE,
];
