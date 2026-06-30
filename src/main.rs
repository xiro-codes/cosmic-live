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

    let mut app = App::new();

    if let Some(wallpaper_path) = cli.wallpaper {
        let colors = extract_colors_from_wallpaper(&wallpaper_path);
        app.insert_resource(colors.clone());
        app.insert_resource(ClearColor(colors.bg));
    } else if let Ok(home) = std::env::var("HOME") {
        let scheme_path = format!("{}/.local/state/caelestia/scheme.json", home);
        let colors = load_scheme_colors(&scheme_path);

        app.insert_resource(colors.clone());
        app.insert_resource(ClearColor(colors.bg));

        app.insert_resource(SchemeWatcher {
            path: scheme_path.clone(),
            last_modified: std::fs::metadata(&scheme_path)
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| std::time::SystemTime::now()),
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        });

        app.add_systems(Update, watch_scheme);
    } else {
        eprintln!("Warning: HOME environment variable not set. Using default colors.");
        let colors = SchemeColors::default();
        app.insert_resource(colors.clone());
        app.insert_resource(ClearColor(colors.bg));
    }

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        }))
        .add_plugins(bevy_live_wallpaper::LiveWallpaperPlugin::default())
        .add_plugins(bevy_ecs_svg::SvgPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_scene, attach_animations_to_svg, scale_svg_to_window, propagate_svg_styles, apply_theme_colors))
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
    bob_speed: f32,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeColor {
    DeepViolet,
    RoyalBlue,
    PastelPink,
    VibrantPurple,
    Indigo,
    LightPurple,
    StarWhite,
}

impl SchemeColors {
    fn get(&self, theme: ThemeColor) -> Color {
        match theme {
            ThemeColor::DeepViolet => self.deep_violet,
            ThemeColor::RoyalBlue => self.royal_blue,
            ThemeColor::PastelPink => self.pastel_pink,
            ThemeColor::VibrantPurple => self.vibrant_purple,
            ThemeColor::Indigo => self.indigo,
            ThemeColor::LightPurple => self.light_purple,
            ThemeColor::StarWhite => self.star_white,
        }
    }
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

fn load_scheme_colors(path: &str) -> SchemeColors {
    let mut colors = SchemeColors::default();

    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            let get_col = |key: &str, fallback: Color| -> Color {
                if let Some(hex) = json
                    .get("colours")
                    .and_then(|c| c.get(key))
                    .and_then(|s| s.as_str())
                {
                    if let Ok(color) = bevy::color::Srgba::hex(hex) {
                        return Color::from(color);
                    }
                }
                fallback
            };

            colors.bg = get_col("crust", colors.bg);
            colors.deep_violet = get_col("base", colors.deep_violet);
            colors.royal_blue = get_col("blue", colors.royal_blue);
            colors.pastel_pink = get_col("pink", colors.pastel_pink);
            colors.vibrant_purple = get_col("mauve", colors.vibrant_purple);
            colors.indigo = get_col("sapphire", colors.indigo);
            colors.light_purple = get_col("lavender", colors.light_purple);
            colors.star_white = get_col("text", colors.star_white);
        }
    }

    colors
}

#[derive(Resource)]
struct SchemeWatcher {
    path: String,
    last_modified: std::time::SystemTime,
    timer: Timer,
}

fn watch_scheme(
    time: Res<Time>,
    mut watcher: ResMut<SchemeWatcher>,
    mut clear_color: ResMut<ClearColor>,
    mut colors_res: ResMut<SchemeColors>,
) {
    if watcher.timer.tick(time.delta()).just_finished() {
        if let Ok(metadata) = std::fs::metadata(&watcher.path) {
            if let Ok(modified) = metadata.modified() {
                if modified > watcher.last_modified {
                    watcher.last_modified = modified;
                    let new_colors = load_scheme_colors(&watcher.path);

                    clear_color.0 = new_colors.bg;
                    *colors_res = new_colors;
                }
            }
        }
    }
}

fn apply_theme_colors(
    colors: Res<SchemeColors>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(&ActiveThemeColor, &MeshMaterial2d<ColorMaterial>)>,
) {
    if colors.is_changed() {
        for (theme_color, mat_handle) in query.iter() {
            if let Some(mut mat) = materials.get_mut(&mat_handle.0) {
                let mut new_c = colors.get(theme_color.0);
                
                // Preserve the alpha if it's currently set (e.g. for twinkling stars)
                let current_alpha = mat.color.alpha();
                new_c.set_alpha(current_alpha);
                mat.color = new_c;
            }
        }
    }
}


fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, bevy_live_wallpaper::LiveWallpaperCamera));

    // Spawn the SVG
    commands.spawn((
        bevy_ecs_svg::SpawnSvg {
            document: asset_server.load("shell.svg"),
        },
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
}

#[derive(Component)]
struct CloudRotate {
    speed: f32,
    phase: f32,
}

#[derive(Component)]
struct SvgStyle {
    color: Option<ThemeColor>,
    shadow: bool,
}

fn attach_animations_to_svg(
    mut commands: Commands,
    query: Query<(Entity, &bevy_ecs_svg::SvgNode), Added<bevy_ecs_svg::SvgNode>>,
) {
    let mut i = 0.0;
    for (entity, node) in query.iter() {
        let id = &node.id;

        if id.starts_with("Star") || id.starts_with("LogoStar") {
            commands.entity(entity).insert((
                StarTwinkle {
                    base_scale: 1.0, // used as base alpha
                    speed: 1.5 + (i % 2.0),
                    phase: i,
                },
                SvgStyle { color: Some(ThemeColor::StarWhite), shadow: true },
            ));
        } else if id.starts_with("Cloud") {
            let cloud_color = if (i as i32) % 2 == 0 { ThemeColor::PastelPink } else { ThemeColor::VibrantPurple };
            commands.entity(entity).insert((
                CloudRotate {
                    speed: 0.1 + (i % 0.1),
                    phase: i,
                },
                SvgStyle { color: Some(cloud_color), shadow: false },
            ));
        } else if id == "Logo" {
            commands.entity(entity).insert((
                PlanetPart {
                    bob_speed: 0.8,
                },
                SvgStyle { color: Some(ThemeColor::StarWhite), shadow: true },
            ));
        } else if id.starts_with("Wave") {
            let wave_color = match id.as_str() {
                "Wave1" => ThemeColor::DeepViolet,
                "Wave2" => ThemeColor::RoyalBlue,
                "Wave3" => ThemeColor::Indigo,
                _ => ThemeColor::LightPurple,
            };
            commands.entity(entity).insert((
                Wave {
                    base_phase: i * 0.5,
                    speed: 0.2 + (i % 0.3),
                },
                SvgStyle { color: Some(wave_color), shadow: false },
            ));
        }
        i += 1.0;
    }
}

#[derive(Component)]
struct ActiveThemeColor(ThemeColor);

fn propagate_svg_styles(
    mut commands: Commands,
    roots: Query<(Entity, &SvgStyle), Added<SvgStyle>>,
    children_q: Query<&Children>,
    paths: Query<(&Mesh2d, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    colors_res: Res<SchemeColors>,
) {
    for (root_entity, style) in roots.iter() {
        let mut queue = vec![root_entity];
        while let Some(entity) = queue.pop() {
            if let Ok((mesh, mat_handle)) = paths.get(entity) {
                if let Some(color_type) = style.color {
                    if let Some(mut mat) = materials.get_mut(&mat_handle.0) {
                        mat.color = colors_res.get(color_type);
                        commands.entity(entity).insert(ActiveThemeColor(color_type));
                    }
                }

                if style.shadow {
                    let shadow_mat = materials.add(ColorMaterial::from(Color::srgba(0.0, 0.0, 0.0, 0.5)));
                    let shadow = commands.spawn((
                        Mesh2d(mesh.0.clone()),
                        MeshMaterial2d(shadow_mat),
                        Transform::from_xyz(10.0, -10.0, -0.1),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                    )).id();
                    commands.entity(entity).add_child(shadow);
                }
            }

            if let Ok(children) = children_q.get(entity) {
                for child in children.iter() {
                    queue.push(child);
                }
            }
        }
    }
}

fn animate_scene(
    time: Res<Time>,
    mut waves: Query<(&mut Transform, &Wave), (Without<StarTwinkle>, Without<PlanetPart>, Without<CloudRotate>)>,
    mut stars: Query<(Entity, &mut Transform, &StarTwinkle), (Without<Wave>, Without<PlanetPart>, Without<CloudRotate>)>,
    mut planets: Query<(&mut Transform, &PlanetPart), (Without<Wave>, Without<StarTwinkle>, Without<CloudRotate>)>,
    mut clouds: Query<(&mut Transform, &CloudRotate), (Without<Wave>, Without<StarTwinkle>, Without<PlanetPart>)>,
    children_q: Query<&Children>,
    paths: Query<&MeshMaterial2d<ColorMaterial>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let t = time.elapsed_secs();

    // Wave should ripple (shift X and Y slightly)
    for (mut transform, wave) in waves.iter_mut() {
        let shift_x = (t * wave.speed + wave.base_phase).sin() * 50.0;
        let shift_y = (t * wave.speed * 1.5 + wave.base_phase).cos() * 15.0;
        transform.translation.x = shift_x;
        transform.translation.y = shift_y;
    }

    // Stars should twinkle (opacity), rotate, and bob
    for (entity, mut transform, star) in stars.iter_mut() {
        let mut queue = vec![entity];
        while let Some(e) = queue.pop() {
            if let Ok(mat_handle) = paths.get(e) {
                if let Some(mut mat) = materials.get_mut(&mat_handle.0) {
                    let alpha = star.base_scale * 0.5 + (t * star.speed * 0.4 + star.phase).sin() * 0.5;
                    mat.color.set_alpha(alpha.clamp(0.0, 1.0));
                }
            }
            if let Ok(children) = children_q.get(e) {
                for child in children.iter() {
                    queue.push(child);
                }
            }
        }
        
        let rotation = (t * star.speed * 0.2 + star.phase).sin() * 0.1;
        let bob = (t * star.speed * 0.3 + star.phase).sin() * 1.5;
        transform.rotation = Quat::from_rotation_z(rotation);
        transform.translation.y = bob;
    }

    // Logo should wobble (bob + rotate)
    for (mut transform, planet) in planets.iter_mut() {
        let bob = (t * planet.bob_speed).sin() * 8.0;
        let wobble = (t * planet.bob_speed * 0.5).sin() * 0.05;
        transform.translation.y = bob;
        transform.rotation = Quat::from_rotation_z(wobble);
    }

    // Clouds should rotate (subtle rotation drift)
    for (mut transform, cloud) in clouds.iter_mut() {
        let rotation = (t * cloud.speed + cloud.phase).sin() * 0.02;
        transform.rotation = Quat::from_rotation_z(rotation);
    }
}

fn scale_svg_to_window(
    mut query: Query<&mut Transform, With<bevy_ecs_svg::SvgHierarchyLoaded>>,
    cameras: Query<&Camera, With<Camera2d>>,
) {
    let Some(camera) = cameras.iter().next() else { return };
    let Some(viewport_size) = camera.logical_viewport_size() else { return };
    
    let cam_w = viewport_size.x;
    let cam_h = viewport_size.y;
    
    if cam_w == 0.0 || cam_h == 0.0 { return; }

    // shell.svg original size is 5120 x 2160
    let svg_w = 5120.0;
    let svg_h = 2160.0;

    // Stretch to fit the resolution perfectly
    let scale_x = cam_w / svg_w;
    let scale_y = cam_h / svg_h;

    for mut transform in query.iter_mut() {
        transform.scale = Vec3::new(scale_x, scale_y, 1.0);
    }
}
