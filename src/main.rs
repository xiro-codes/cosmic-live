use bevy::prelude::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    wallpaper: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let colors = cli.wallpaper.as_ref().map(|p| extract_colors_from_wallpaper(p)).unwrap_or_default();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        }))
        .add_plugins(bevy_live_wallpaper::LiveWallpaperPlugin::default())
        .insert_resource(colors.clone())
        .insert_resource(ClearColor(colors.bg)) // Scheme background
        .add_systems(Startup, setup)
        .add_systems(Update, animate_scene)
        .run();
}

#[derive(Component)]
struct Wave {
    base_phase: f32,
    speed: f32,
}

#[derive(Component)]
struct StarTwinkle {
    base_scale: f32,
    speed: f32,
    phase: f32,
}

#[derive(Component)]
struct PlanetPart {
    base_y: f32,
    base_rotation: f32,
    bob_speed: f32,
    spin_speed: f32,
}

#[derive(Resource, Clone)]
pub struct SchemeColors {
    bg: Color,
    deep_violet: Color,
    royal_blue: Color,
    pastel_pink: Color,
    vibrant_purple: Color,
    indigo: Color,
    light_purple: Color,
    star_white: Color,
}

impl Default for SchemeColors {
    fn default() -> Self {
        Self {
            bg: Color::srgb(0.05, 0.05, 0.1),
            deep_violet: Color::srgb(0.1, 0.05, 0.2),
            royal_blue: Color::srgb(0.1, 0.15, 0.4),
            pastel_pink: Color::srgb(0.8, 0.5, 0.7),
            vibrant_purple: Color::srgb(0.4, 0.1, 0.5),
            indigo: Color::srgb(0.15, 0.1, 0.3),
            light_purple: Color::srgb(0.5, 0.3, 0.6),
            star_white: Color::WHITE,
        }
    }
}

fn extract_colors_from_wallpaper(path: &str) -> SchemeColors {
    let mut colors = SchemeColors::default();

    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(dyn_img) = image::load_from_memory(&bytes) {
            let img_resized = dyn_img.resize_exact(32, 32, image::imageops::FilterType::Triangle);
            let mut pixels: Vec<_> = img_resized.to_rgba8().pixels().map(|p| p.0).collect();

            pixels.sort_by(|a, b| {
                let max_a = a[0].max(a[1]).max(a[2]) as f32;
                let min_a = a[0].min(a[1]).min(a[2]) as f32;
                let sat_a = if max_a == 0.0 { 0.0 } else { (max_a - min_a) / max_a };

                let max_b = b[0].max(b[1]).max(b[2]) as f32;
                let min_b = b[0].min(b[1]).min(b[2]) as f32;
                let sat_b = if max_b == 0.0 { 0.0 } else { (max_b - min_b) / max_b };

                sat_b.partial_cmp(&sat_a).unwrap_or(std::cmp::Ordering::Equal)
            });

            let mut chosen_colors = Vec::new();
            let mut target_dist = 0.5;

            while chosen_colors.len() < 8 && target_dist >= 0.0 {
                for p in &pixels {
                    let color = Color::srgba(
                        p[0] as f32 / 255.0,
                        p[1] as f32 / 255.0,
                        p[2] as f32 / 255.0,
                        1.0,
                    );

                    let mut similar = false;
                    for c in &chosen_colors {
                        let c: Color = *c;
                        let srgba1 = color.to_srgba();
                        let srgba2 = c.to_srgba();
                        let dist = (srgba1.red - srgba2.red).abs()
                            + (srgba1.green - srgba2.green).abs()
                            + (srgba1.blue - srgba2.blue).abs();
                        if dist < target_dist {
                            similar = true;
                            break;
                        }
                    }

                    if !similar {
                        chosen_colors.push(color);
                        if chosen_colors.len() == 8 {
                            break;
                        }
                    }
                }
                target_dist -= 0.05;
            }

            if chosen_colors.len() == 8 {
                colors.bg = chosen_colors[0];
                colors.deep_violet = chosen_colors[1];
                colors.royal_blue = chosen_colors[2];
                colors.pastel_pink = chosen_colors[3];
                colors.vibrant_purple = chosen_colors[4];
                colors.indigo = chosen_colors[5];
                colors.light_purple = chosen_colors[6];
                colors.star_white = chosen_colors[7];
            } else {
                for (i, c) in chosen_colors.iter().enumerate() {
                    match i {
                        0 => colors.bg = *c,
                        1 => colors.deep_violet = *c,
                        2 => colors.royal_blue = *c,
                        3 => colors.pastel_pink = *c,
                        4 => colors.vibrant_purple = *c,
                        5 => colors.indigo = *c,
                        6 => colors.light_purple = *c,
                        7 => colors.star_white = *c,
                        _ => {}
                    }
                }
            }
        }
    }

    colors
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    colors: Res<SchemeColors>,
) {
    commands.spawn((Camera2d, bevy_live_wallpaper::LiveWallpaperCamera));

    let deep_violet_mat = materials.add(ColorMaterial::from(colors.deep_violet));
    let royal_blue_mat = materials.add(ColorMaterial::from(colors.royal_blue));
    let pastel_pink_mat = materials.add(ColorMaterial::from(colors.pastel_pink));
    let vibrant_purple_mat = materials.add(ColorMaterial::from(colors.vibrant_purple));
    let indigo_mat = materials.add(ColorMaterial::from(colors.indigo));
    let light_purple_mat = materials.add(ColorMaterial::from(colors.light_purple));
    let star_white_mat = materials.add(ColorMaterial::from(colors.star_white));
    let shadow_color = Color::srgba(0.0, 0.0, 0.0, 0.6);

    let screen_w = 4000.0;
    let bottom_y = -2000.0;

    // Background Waves (from back to front)
    let wave_data = vec![
        (deep_violet_mat, 800.0, 150.0, 1.5, 0.0, 0.2, -10.0),
        (royal_blue_mat, 400.0, 200.0, 2.0, 1.0, 0.3, -9.0),
        (pastel_pink_mat.clone(), 0.0, 100.0, 1.0, 2.0, 0.15, -8.0),
        (vibrant_purple_mat, -300.0, 180.0, 2.5, 0.5, 0.25, -7.0),
        (indigo_mat, -600.0, 250.0, 1.2, 1.5, 0.2, -6.0),
        (light_purple_mat, -900.0, 120.0, 1.8, 0.8, 0.1, -5.0),
    ];

    for (mat, base_y, amp, freq, phase, speed, z) in wave_data {
        commands.spawn((
            Mesh2d(meshes.add(create_wave_mesh(
                screen_w, base_y, amp, freq, phase, bottom_y,
            ))),
            MeshMaterial2d(mat),
            Transform::from_xyz(0.0, 0.0, z),
            Wave {
                base_phase: phase,
                speed,
            },
        ));
    }

    // Central Planet Logo
    let planet_z = 0.0;

    let top_crescent_mesh = meshes.add(create_crescent_mesh(true));
    let bot_crescent_mesh = meshes.add(create_crescent_mesh(false));
    let ring_mesh = meshes.add(create_custom_ring_mesh());

    let shadow_mat = materials.add(ColorMaterial::from(shadow_color));

    // Shadows
    commands.spawn((
        Mesh2d(top_crescent_mesh.clone()),
        MeshMaterial2d(shadow_mat.clone()),
        Transform::from_xyz(15.0, -15.0, planet_z - 0.5),
        PlanetPart {
            base_y: -15.0,
            base_rotation: 0.0,
            bob_speed: 0.8,
            spin_speed: 0.05,
        },
    ));
    commands.spawn((
        Mesh2d(bot_crescent_mesh.clone()),
        MeshMaterial2d(shadow_mat.clone()),
        Transform::from_xyz(15.0, -15.0, planet_z - 0.5),
        PlanetPart {
            base_y: -15.0,
            base_rotation: 0.0,
            bob_speed: 0.8,
            spin_speed: 0.05,
        },
    ));
    commands.spawn((
        Mesh2d(ring_mesh.clone()),
        MeshMaterial2d(shadow_mat),
        Transform::from_xyz(15.0, -15.0, planet_z - 0.5),
        PlanetPart {
            base_y: -15.0,
            base_rotation: 0.0,
            bob_speed: 0.8,
            spin_speed: 0.05,
        },
    ));

    // Top Pink Crescent
    commands.spawn((
        Mesh2d(top_crescent_mesh),
        MeshMaterial2d(pastel_pink_mat.clone()),
        Transform::from_xyz(0.0, 0.0, planet_z),
        PlanetPart {
            base_y: 0.0,
            base_rotation: 0.0,
            bob_speed: 0.8,
            spin_speed: 0.05,
        },
    ));

    // Bottom Pink Crescent
    commands.spawn((
        Mesh2d(bot_crescent_mesh),
        MeshMaterial2d(pastel_pink_mat.clone()),
        Transform::from_xyz(0.0, 0.0, planet_z),
        PlanetPart {
            base_y: 0.0,
            base_rotation: 0.0,
            bob_speed: 0.8,
            spin_speed: 0.05,
        },
    ));

    // White Ring (Tilted C-shape)
    commands.spawn((
        Mesh2d(ring_mesh),
        MeshMaterial2d(star_white_mat.clone()),
        Transform::from_xyz(0.0, 0.0, planet_z + 1.0),
        PlanetPart {
            base_y: 0.0,
            base_rotation: 0.0,
            bob_speed: 0.8,
            spin_speed: 0.05,
        },
    ));

    // Stars
    let stars = vec![
        // Left side
        (star_white_mat.clone(), 40.0, -300.0, -200.0, 2.0, 0.0),
        (pastel_pink_mat.clone(), 20.0, -500.0, 100.0, 1.5, 1.0),
        (pastel_pink_mat.clone(), 25.0, -250.0, 300.0, 3.0, 2.0),
        (pastel_pink_mat.clone(), 10.0, -150.0, 150.0, 2.5, 3.0),
        // Right side
        (star_white_mat.clone(), 50.0, 350.0, 250.0, 1.8, 0.5),
        (pastel_pink_mat.clone(), 15.0, 180.0, 220.0, 4.0, 1.5),
        (pastel_pink_mat.clone(), 12.0, 220.0, 280.0, 3.5, 2.5),
        (pastel_pink_mat.clone(), 20.0, 450.0, -100.0, 2.2, 0.8),
        (pastel_pink_mat.clone(), 18.0, 300.0, -350.0, 1.6, 1.2),
    ];

    let star_shadow_mat = materials.add(ColorMaterial::from(shadow_color));

    for (mat, radius, x, y, speed, phase) in stars {
        let star_mesh = meshes.add(create_star_mesh(radius));

        // Star Shadow
        commands.spawn((
            Mesh2d(star_mesh.clone()),
            MeshMaterial2d(star_shadow_mat.clone()),
            Transform::from_xyz(x + 5.0, y - 5.0, planet_z + 1.9),
            StarTwinkle {
                base_scale: 1.0,
                speed,
                phase,
            },
        ));

        // Actual Star
        commands.spawn((
            Mesh2d(star_mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(x, y, planet_z + 2.0),
            StarTwinkle {
                base_scale: 1.0,
                speed,
                phase,
            },
        ));
    }

}

fn animate_scene(
    time: Res<Time>,
    mut waves: Query<(&mut Transform, &Wave), (Without<StarTwinkle>, Without<PlanetPart>)>,
    mut stars: Query<(&mut Transform, &StarTwinkle), (Without<Wave>, Without<PlanetPart>)>,
    mut planets: Query<(&mut Transform, &PlanetPart), (Without<Wave>, Without<StarTwinkle>)>,
) {
    let t = time.elapsed_secs();

    // Animate waves by slowly shifting them horizontally
    for (mut transform, wave) in waves.iter_mut() {
        let shift = (t * wave.speed + wave.base_phase).sin() * 50.0;
        transform.translation.x = shift;
    }

    // Twinkle and spin stars
    for (mut transform, star) in stars.iter_mut() {
        let scale = star.base_scale + (t * star.speed + star.phase).sin() * 0.2;
        transform.scale = Vec3::splat(scale);
        transform.rotation = Quat::from_rotation_z(t * 0.5 * star.speed);
    }

    // Bob and spin planet parts
    for (mut transform, planet) in planets.iter_mut() {
        transform.translation.y = planet.base_y + (t * planet.bob_speed).sin() * 15.0;
        transform.rotation = Quat::from_rotation_z(planet.base_rotation + t * planet.spin_speed);
    }
}

// Custom Mesh Generators

fn create_star_mesh(radius: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let segments = 32;

    positions.push([0.0, 0.0, 0.0]); // Center

    for i in 0..segments {
        let t = (i as f32) * std::f32::consts::TAU / (segments as f32);
        let x = radius * t.cos().powi(3);
        let y = radius * t.sin().powi(3);
        positions.push([x, y, 0.0]);
    }

    for i in 0..segments {
        let a = 0;
        let b = i + 1;
        let c = if i == segments - 1 { 1 } else { i + 2 };
        indices.push(a);
        indices.push(b);
        indices.push(c);
    }

    Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}

fn create_crescent_mesh(is_top: bool) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let segments = 200;
    let r = 80.0_f32;

    let a = 160.0_f32;
    let b = 50.0_f32;
    let alpha = 22.0_f32.to_radians();

    let big_a = alpha.sin().powi(2) / a.powi(2) + alpha.cos().powi(2) / b.powi(2);

    for i in 0..=segments {
        let x = -r + 2.0 * r * (i as f32) / (segments as f32);
        let y_circ = (r.powi(2) - x.powi(2)).max(0.0).sqrt();

        let big_b = 2.0 * x * alpha.sin() * alpha.cos() * (1.0 / a.powi(2) - 1.0 / b.powi(2));
        let big_c = x.powi(2) * alpha.cos().powi(2) / a.powi(2)
            + x.powi(2) * alpha.sin().powi(2) / b.powi(2)
            - 1.0;

        let disc = big_b.powi(2) - 4.0 * big_a * big_c;
        let mut y_ell_top = 0.0;
        let mut y_ell_bot = 0.0;

        if disc >= 0.0 {
            y_ell_top = (-big_b + disc.sqrt()) / (2.0 * big_a);
            y_ell_bot = (-big_b - disc.sqrt()) / (2.0 * big_a);
        }

        if is_top {
            if y_ell_top <= y_circ {
                positions.push([x, y_ell_top, 0.0]);
                positions.push([x, y_circ, 0.0]);
            } else {
                positions.push([x, y_circ, 0.0]);
                positions.push([x, y_circ, 0.0]);
            }
        } else {
            if -y_circ <= y_ell_bot {
                positions.push([x, -y_circ, 0.0]);
                positions.push([x, y_ell_bot, 0.0]);
            } else {
                positions.push([x, -y_circ, 0.0]);
                positions.push([x, -y_circ, 0.0]);
            }
        }
    }

    for i in 0..segments {
        let p1 = i * 2;
        let p2 = i * 2 + 1;
        let p3 = i * 2 + 2;
        let p4 = i * 2 + 3;

        indices.extend_from_slice(&[p1, p2, p3]);
        indices.extend_from_slice(&[p3, p2, p4]);
    }

    Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}

fn create_custom_ring_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let segments = 200;

    let a_in = 130.0;
    let b_in = 20.0;
    let a_out = 150.0;
    let b_out = 40.0;
    let alpha = 22.0_f32.to_radians();

    let t_start = 20.0_f32.to_radians();
    let t_end = 340.0_f32.to_radians();

    for i in 0..=segments {
        let t = t_start + (t_end - t_start) * (i as f32) / (segments as f32);

        let u_in = a_in * t.cos();
        let v_in = b_in * t.sin();
        let mut u_out = a_out * t.cos();
        let v_out = b_out * t.sin();

        let dist_to_pi = (t - std::f32::consts::PI).abs();
        if dist_to_pi < 0.5 {
            let wing = (1.0 - dist_to_pi / 0.5).powi(2) * 80.0;
            u_out -= wing;
        }

        let x_in = u_in * alpha.cos() - v_in * alpha.sin();
        let y_in = u_in * alpha.sin() + v_in * alpha.cos();
        let x_out = u_out * alpha.cos() - v_out * alpha.sin();
        let y_out = u_out * alpha.sin() + v_out * alpha.cos();

        positions.push([x_out, y_out, 0.0]);
        positions.push([x_in, y_in, 0.0]);
    }

    for i in 0..segments {
        let p1 = i * 2;
        let p2 = i * 2 + 1;
        let p3 = i * 2 + 2;
        let p4 = i * 2 + 3;

        indices.extend_from_slice(&[p1, p2, p3]);
        indices.extend_from_slice(&[p3, p2, p4]);
    }

    Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}

fn create_wave_mesh(
    width: f32,
    base_y: f32,
    amplitude: f32,
    frequency: f32,
    phase: f32,
    bottom_y: f32,
) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let segments = 100;

    for i in 0..=segments {
        let normalized_x = (i as f32) / (segments as f32);
        let x = -width / 2.0 + normalized_x * width;
        let wave_y =
            base_y + amplitude * (frequency * normalized_x * std::f32::consts::TAU + phase).sin();

        positions.push([x, wave_y, 0.0]);
        positions.push([x, bottom_y, 0.0]);
    }

    for i in 0..segments {
        let p1 = i * 2;
        let p2 = i * 2 + 1;
        let p3 = i * 2 + 2;
        let p4 = i * 2 + 3;

        indices.extend_from_slice(&[p1, p2, p3]);
        indices.extend_from_slice(&[p3, p2, p4]);
    }

    Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}
