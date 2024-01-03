use std::thread::current;

use bevy::{
    math::DVec2,
    prelude::*,
    sprite::MaterialMesh2dBundle,
    tasks::{AsyncComputeTaskPool, Task},
};

pub struct GenerationPlugin;

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init)
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

#[derive(Debug)]
struct CurrentZoom {
    center: DVec2,
    extent: DVec2,
}

impl CurrentZoom {
    fn from_window(window: &Window) -> Self {
        let window_resolution = &window.resolution;

        let width = window_resolution.width() as f64;
        let height = window_resolution.height() as f64;
        let extent = if width > height {
            DVec2::new(1.5, 1.5 * height / width)
        } else {
            DVec2::new(width / height, 1.0)
        };

        Self {
            center: DVec2::new(-0.5, 0.0),
            extent,
        }
    }
}

fn init(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(shape::Quad::new(CELL_SIZE * Vec2::ONE).into());

    commands.insert_resource(GenerateAssets { mesh });
    commands.insert_resource(RegenerateTimer(Timer::from_seconds(0.25, TimerMode::Once)));
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

struct MandelbrotResult(Vec<Color>);

#[derive(Component)]
struct MandelbrotTask(Task<MandelbrotResult>);

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

        let window = window.single();
        let window_resolution = &window.resolution;
        let width = window_resolution.width();
        let height = window_resolution.height();

        let cells_x = (width / CELL_SIZE).ceil() as i32;
        let cells_y = (height / CELL_SIZE).ceil() as i32;

        let window_size = 0.5 * Vec2::new(width, height).as_dvec2();
        let cell_origin = -0.5f32 * Vec2::new(width, height);

        let current_zoom = CurrentZoom::from_window(window);
        info!(
            "generating {}x{} ({} {} {:?})",
            cells_x, cells_y, width, height, current_zoom
        );

        let mut parent = commands.spawn((ResultContainer, SpatialBundle::default()));

        let thread_pool = AsyncComputeTaskPool::get();

        (0..cells_x).for_each(|x| (0..cells_y).for_each(|y| {}));
        parent.with_children(|cb| {
            (0..cells_x).for_each(|x| {
                (0..cells_y).for_each(|y| {
                    let color_index = (x + cells_x * y) as usize % COLORS.len();
                    let x = x as f32;
                    let y = y as f32;
                    let min = (cell_origin + CELL_SIZE * Vec2::new(x, y)).as_dvec2();
                    let max = (cell_origin + CELL_SIZE * Vec2::new(x + 1.0, y + 1.0)).as_dvec2();

                    let min_pos = (min / window_size) * current_zoom.extent + current_zoom.center;
                    let max_pos = (max / window_size) * current_zoom.extent + current_zoom.center;

                    info!("{} {}", min_pos, max_pos);
                    let x = (x + 0.5) * CELL_SIZE;
                    let y = (y + 0.5) * CELL_SIZE;
                    let trans = cell_origin + Vec2::new(x, y);

                    let task = thread_pool.spawn(async move { MandelbrotResult(vec![]) });

                    cb.spawn(MandelbrotTask(task));

                    // cb.spawn(MaterialMesh2dBundle {
                    //     mesh: assets.mesh.clone().into(),
                    //     material: materials.add(ColorMaterial::from(COLORS[color_index])),
                    //     transform: Transform::from_translation(trans.extend(0.0)),
                    //     ..default()
                    // });
                });
            });
        });

        commands.remove_resource::<GenerateCommand>();
    }
}

const CELL_SIZE: f32 = 256.0f32;

const COLORS: [bevy::prelude::Color; 6] = [
    Color::ALICE_BLUE,
    Color::ORANGE_RED,
    Color::TURQUOISE,
    Color::DARK_GREEN,
    Color::VIOLET,
    Color::BISQUE,
];
