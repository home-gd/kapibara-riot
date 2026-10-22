use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::level::{Level, Surface, spawn_level};

const MOVE_SPEED: f32 = 400.0;
const JUMP_SPEED: f32 = 780.0;
const GRAVITY: f32 = 2200.0;
/// Apex of a press that is released immediately, measured above the surface jumped from.
const MIN_JUMP_HEIGHT: f32 = 110.0;
/// Highest the rise can go while the jump key stays held, above the takeoff surface.
const MAX_JUMP_HEIGHT: f32 = 300.0;
const PLAYER_SCALE: f32 = 4.0;
const FRAME_SECS: f32 = 0.15;
/// `movement_animation.png` is 192×64: three 64×64 frames. Index 0 is the still pose.
const MOVEMENT_LAST: usize = 2;
/// `jump_animation.png` is 128×64: two 64×64 frames. Play once, then hold the last frame in the air.
const JUMP_LAST: usize = 1;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.after(spawn_level))
            .add_systems(
                Update,
                (player_movement, animate_sprite, follow_camera).chain(),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Clip {
    Movement,
    Jump,
}

#[derive(Component)]
struct Player {
    velocity_y: f32,
    /// Set once this jump stops climbing, so a later press in the air cannot extend it.
    rise_ended: bool,
    jump_origin: f32,
    /// One-way surface top being passed through after Down/S + Space.
    drop_top: Option<f32>,
    clip: Clip,
}

#[derive(Component)]
struct PlayerClips {
    movement_image: Handle<Image>,
    movement_layout: Handle<TextureAtlasLayout>,
    jump_image: Handle<Image>,
    jump_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn setup(
    mut commands: Commands,
    level: Res<Level>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let movement_image = asset_server.load("player/movement_animation.png");
    let jump_image = asset_server.load("player/jump_animation.png");
    let movement_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(64),
        3,
        1,
        None,
        None,
    ));
    let jump_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(64),
        2,
        1,
        None,
        None,
    ));

    commands.spawn((
        Player {
            velocity_y: 0.0,
            rise_ended: false,
            jump_origin: level.spawn.y,
            drop_top: None,
            clip: Clip::Movement,
        },
        PlayerClips {
            movement_image: movement_image.clone(),
            movement_layout: movement_layout.clone(),
            jump_image,
            jump_layout,
        },
        Sprite::from_atlas_image(
            movement_image,
            TextureAtlas {
                layout: movement_layout,
                index: 0,
            },
        ),
        Transform::from_xyz(level.spawn.x, level.spawn.y, 2.0)
            .with_scale(Vec3::splat(PLAYER_SCALE)),
        Anchor::BOTTOM_CENTER,
        AnimationTimer(Timer::from_seconds(FRAME_SECS, TimerMode::Repeating)),
    ));
}

/// Net horizontal input. Left and right cancel, including mixes such as A + ArrowRight.
fn horizontal_axis(keyboard: &ButtonInput<KeyCode>) -> f32 {
    let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);
    let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    match (left, right) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

fn jump_input(keyboard: &ButtonInput<KeyCode>) -> (bool, bool) {
    const KEYS: [KeyCode; 3] = [KeyCode::Space, KeyCode::ArrowUp, KeyCode::KeyW];
    let started = KEYS.iter().any(|key| keyboard.just_pressed(*key));
    let held = KEYS.iter().any(|key| keyboard.pressed(*key));
    (started, held)
}

/// Down or S, together with a new Space press, drops through a platform.
fn drop_requested(keyboard: &ButtonInput<KeyCode>) -> bool {
    let down = keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown);
    down && keyboard.just_pressed(KeyCode::Space)
}

struct Vertical {
    y: f32,
    velocity_y: f32,
    jumping: bool,
    rise_ended: bool,
    jump_origin: f32,
    drop_top: Option<f32>,
}

fn overlaps_x(surface: &Surface, x: f32) -> bool {
    x >= surface.left && x <= surface.right
}

fn surface_under<'a>(surfaces: &'a [Surface], x: f32, y: f32) -> Option<&'a Surface> {
    const SLOP: f32 = 0.8;
    surfaces
        .iter()
        .filter(|surface| overlaps_x(surface, x) && (y - surface.top).abs() <= SLOP)
        .max_by(|a, b| a.top.total_cmp(&b.top))
}

/// A new jump starts only while standing on a surface. Heights are relative to that
/// surface. Releasing before the short hop still finishes it. Holding climbs toward
/// the cap. Releasing after the short hop, or hitting the cap, starts the fall.
/// Down/S + Space on a one-way surface skips that surface; the ground still catches you.
fn step_vertical(
    body: &mut Vertical,
    x: f32,
    started: bool,
    held: bool,
    drop: bool,
    dt: f32,
    surfaces: &[Surface],
) {
    let under = surface_under(surfaces, x, body.y);
    let grounded = under.is_some() && !body.jumping && body.velocity_y <= 0.0;

    if let Some(surface) = under {
        if grounded && drop && surface.one_way {
            body.drop_top = Some(surface.top);
            body.jumping = true;
            body.rise_ended = true;
            body.velocity_y = 0.0;
        } else if grounded && started {
            body.jumping = true;
            body.rise_ended = false;
            body.jump_origin = body.y;
            body.velocity_y = JUMP_SPEED;
            body.drop_top = None;
        }
    }

    let prev_y = body.y;
    let mut powered = false;
    if body.jumping && !body.rise_ended {
        let height = if held {
            MAX_JUMP_HEIGHT
        } else {
            MIN_JUMP_HEIGHT
        };
        let target = body.jump_origin + height;
        if body.y >= target {
            body.velocity_y = 0.0;
            body.rise_ended = true;
        } else {
            let step = JUMP_SPEED * dt;
            if body.y + step >= target {
                body.y = target;
                body.velocity_y = 0.0;
                body.rise_ended = true;
            } else {
                body.y += step;
                body.velocity_y = JUMP_SPEED;
                powered = true;
            }
        }
    }

    if !powered {
        body.velocity_y -= GRAVITY * dt;
        body.y += body.velocity_y * dt;
    }

    if body.velocity_y > 0.0 {
        return;
    }

    let lock = body.drop_top;
    let landing = surfaces
        .iter()
        .filter(|surface| {
            if !overlaps_x(surface, x) {
                return false;
            }
            if let Some(lock_top) = lock {
                if surface.one_way && (surface.top - lock_top).abs() <= 1.0 {
                    return false;
                }
            }
            prev_y >= surface.top && body.y <= surface.top
        })
        .max_by(|a, b| a.top.total_cmp(&b.top));

    if let Some(surface) = landing {
        body.y = surface.top;
        body.velocity_y = 0.0;
        body.jumping = false;
        body.rise_ended = false;
        body.drop_top = None;
    } else if lock.is_some_and(|lock_top| prev_y < lock_top) {
        body.drop_top = None;
    }
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    level: Res<Level>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let axis = horizontal_axis(&keyboard);
    let (jump_started, jump_held) = jump_input(&keyboard);
    let drop = drop_requested(&keyboard);
    let half_width = 64.0 * PLAYER_SCALE * 0.5;

    for (mut player, mut transform) in &mut query {
        if axis != 0.0 {
            transform.translation.x += axis * MOVE_SPEED * dt;
        }
        if let Some(ground) = level.surfaces.iter().find(|surface| !surface.one_way) {
            let min_x = ground.left + half_width;
            let max_x = ground.right - half_width;
            if min_x < max_x {
                transform.translation.x = transform.translation.x.clamp(min_x, max_x);
            }
        }

        let mut vertical = Vertical {
            y: transform.translation.y,
            velocity_y: player.velocity_y,
            jumping: player.clip == Clip::Jump,
            rise_ended: player.rise_ended,
            jump_origin: player.jump_origin,
            drop_top: player.drop_top,
        };
        step_vertical(
            &mut vertical,
            transform.translation.x,
            jump_started,
            jump_held,
            drop,
            dt,
            &level.surfaces,
        );
        transform.translation.y = vertical.y;
        player.velocity_y = vertical.velocity_y;
        player.rise_ended = vertical.rise_ended;
        player.jump_origin = vertical.jump_origin;
        player.drop_top = vertical.drop_top;
        player.clip = if vertical.jumping {
            Clip::Jump
        } else {
            Clip::Movement
        };
    }
}

fn follow_camera(
    level: Res<Level>,
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    view: Query<&Camera, With<Camera2d>>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    let view_size = view
        .single()
        .ok()
        .and_then(|camera| camera.logical_viewport_size())
        .unwrap_or(Vec2::new(1280.0, 720.0));
    let half = view_size * 0.5;
    // Keep the feet a little below center so the pads above them stay in frame.
    let mut x = player.translation.x;
    let mut y = player.translation.y + 140.0;
    let min_x = level.bounds.min.x + half.x;
    let max_x = level.bounds.max.x - half.x;
    let min_y = level.bounds.min.y + half.y;
    let max_y = level.bounds.max.y - half.y;
    if min_x <= max_x {
        x = x.clamp(min_x, max_x);
    }
    if min_y <= max_y {
        y = y.clamp(min_y, max_y);
    }
    let z = camera.translation.z;
    camera.translation = Vec3::new(x, y, z);
}

fn show_clip(sprite: &mut Sprite, image: Handle<Image>, layout: Handle<TextureAtlasLayout>) {
    sprite.image = image;
    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.layout = layout;
        atlas.index = 0;
    }
}

fn animate_sprite(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &PlayerClips, &mut AnimationTimer, &mut Sprite)>,
) {
    let is_moving = horizontal_axis(&keyboard) != 0.0;

    for (player, clips, mut timer, mut sprite) in &mut query {
        match player.clip {
            Clip::Jump => {
                if sprite.image != clips.jump_image {
                    show_clip(
                        &mut sprite,
                        clips.jump_image.clone(),
                        clips.jump_layout.clone(),
                    );
                    timer.reset();
                }

                timer.tick(time.delta());
                if timer.just_finished() {
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        if atlas.index < JUMP_LAST {
                            atlas.index += 1;
                        }
                    }
                }
            }
            Clip::Movement => {
                if sprite.image != clips.movement_image {
                    show_clip(
                        &mut sprite,
                        clips.movement_image.clone(),
                        clips.movement_layout.clone(),
                    );
                    timer.reset();
                }

                let Some(atlas) = &mut sprite.texture_atlas else {
                    continue;
                };

                if !is_moving {
                    atlas.index = 0;
                    timer.reset();
                    continue;
                }

                timer.tick(time.delta());
                if timer.just_finished() {
                    atlas.index = if atlas.index == MOVEMENT_LAST {
                        0
                    } else {
                        atlas.index + 1
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposing_horizontal_keys_cancel() {
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyA);
        keys.press(KeyCode::ArrowRight);
        assert_eq!(horizontal_axis(&keys), 0.0);

        keys = ButtonInput::default();
        keys.press(KeyCode::KeyD);
        assert_eq!(horizontal_axis(&keys), 1.0);
    }

    fn grounded() -> Vertical {
        Vertical {
            y: 0.0,
            velocity_y: 0.0,
            jumping: false,
            rise_ended: false,
            jump_origin: 0.0,
            drop_top: None,
        }
    }

    fn floor() -> [Surface; 1] {
        [Surface {
            left: -10_000.0,
            right: 10_000.0,
            top: 0.0,
            one_way: false,
        }]
    }

    fn step(
        body: &mut Vertical,
        started: bool,
        held: bool,
        drop: bool,
        dt: f32,
        surfaces: &[Surface],
    ) {
        step_vertical(body, 0.0, started, held, drop, dt, surfaces);
    }

    fn peak_of(held_frames: usize) -> f32 {
        let mut body = grounded();
        let mut peak: f32 = 0.0;
        let mut started = true;
        for frame in 0..360 {
            let held = frame < held_frames;
            step(&mut body, started, held, false, 1.0 / 60.0, &floor());
            started = false;
            peak = peak.max(body.y);
            if frame > 0 && !body.jumping {
                break;
            }
        }
        peak
    }

    #[test]
    fn rest_stays_on_ground() {
        let mut body = grounded();
        let floor = floor();
        for _ in 0..10 {
            step(&mut body, false, false, false, 1.0 / 60.0, &floor);
        }
        assert_eq!(body.y, 0.0);
        assert_eq!(body.velocity_y, 0.0);
        assert!(!body.jumping);
    }

    #[test]
    fn tap_reaches_the_short_hop_and_lands() {
        let peak = peak_of(1);
        assert!((peak - MIN_JUMP_HEIGHT).abs() < 2.0, "peak {peak}");

        let mut body = grounded();
        let floor = floor();
        step(&mut body, true, true, false, 1.0 / 60.0, &floor);
        let mut landed = false;
        for _ in 0..180 {
            step(&mut body, false, false, false, 1.0 / 60.0, &floor);
            if !body.jumping {
                landed = true;
                break;
            }
        }
        assert!(landed);
        assert_eq!(body.y, 0.0);
        assert_eq!(body.velocity_y, 0.0);
    }

    #[test]
    fn holding_jumps_higher_but_not_past_the_cap() {
        let tap = peak_of(1);
        let held = peak_of(360);
        assert!(held > tap + 100.0, "tap {tap}, held {held}");
        assert!((held - MAX_JUMP_HEIGHT).abs() < 2.0, "held peak {held}");
    }

    #[test]
    fn release_after_the_short_hop_starts_the_drop() {
        let mut body = grounded();
        let floor = floor();
        let mut started = true;
        for _ in 0..180 {
            step(&mut body, started, true, false, 1.0 / 60.0, &floor);
            started = false;
            if body.y > MIN_JUMP_HEIGHT + 30.0 {
                break;
            }
        }
        assert!(body.y < MAX_JUMP_HEIGHT);
        let y_before_release = body.y;
        step(&mut body, false, false, false, 1.0 / 60.0, &floor);
        assert!(body.velocity_y < 0.0);
        assert!(body.y < y_before_release);
        assert!(body.rise_ended);
    }

    #[test]
    fn press_again_in_the_air_does_not_extend_or_restart() {
        let mut body = grounded();
        let floor = floor();
        step(&mut body, true, true, false, 1.0 / 60.0, &floor);
        for _ in 0..180 {
            step(&mut body, false, false, false, 1.0 / 60.0, &floor);
            if body.rise_ended && body.jumping {
                break;
            }
        }
        assert!(body.jumping);
        let y = body.y;
        step(&mut body, true, true, false, 1.0 / 60.0, &floor);
        assert!(body.y < y);
        assert!(body.velocity_y < 0.0);
    }

    #[test]
    fn holding_through_landing_does_not_bounce() {
        let mut body = grounded();
        let floor = floor();
        let mut started = true;
        for _ in 0..360 {
            step(&mut body, started, true, false, 1.0 / 60.0, &floor);
            started = false;
            if !body.jumping {
                break;
            }
        }
        assert_eq!(body.y, 0.0);
        step(&mut body, false, true, false, 1.0 / 60.0, &floor);
        assert_eq!(body.y, 0.0);
        assert!(!body.jumping);
    }

    fn platform_level() -> Vec<Surface> {
        vec![
            Surface {
                left: -100.0,
                right: 100.0,
                top: 200.0,
                one_way: true,
            },
            Surface {
                left: -1_000.0,
                right: 1_000.0,
                top: 0.0,
                one_way: false,
            },
        ]
    }

    #[test]
    fn falling_lands_on_a_platform() {
        let surfaces = platform_level();
        let mut body = grounded();
        body.y = 400.0;
        body.jumping = true;
        for _ in 0..240 {
            step(&mut body, false, false, false, 1.0 / 60.0, &surfaces);
            if !body.jumping {
                break;
            }
        }
        assert_eq!(body.y, 200.0);
        assert!(!body.jumping);
    }

    #[test]
    fn jump_height_is_relative_to_the_platform() {
        let surfaces = platform_level();
        let mut body = grounded();
        body.y = 200.0;
        let mut peak = body.y;
        let mut started = true;
        for frame in 0..180 {
            step(&mut body, started, frame == 0, false, 1.0 / 60.0, &surfaces);
            started = false;
            peak = peak.max(body.y);
            if frame > 0 && !body.jumping {
                break;
            }
        }
        assert!(
            (peak - (200.0 + MIN_JUMP_HEIGHT)).abs() < 2.0,
            "peak {peak}"
        );
        assert_eq!(body.y, 200.0);
    }

    #[test]
    fn down_and_space_drops_through_a_platform_onto_the_ground() {
        let surfaces = platform_level();
        let mut body = grounded();
        body.y = 200.0;
        step(&mut body, true, true, true, 1.0 / 60.0, &surfaces);
        assert!(body.y < 200.0, "y {}", body.y);
        assert!(body.jumping);
        let mut landed = false;
        for _ in 0..240 {
            step(&mut body, false, false, false, 1.0 / 60.0, &surfaces);
            if !body.jumping {
                landed = true;
                break;
            }
        }
        assert!(landed);
        assert_eq!(body.y, 0.0);
    }

    #[test]
    fn down_and_space_on_the_ground_still_jumps() {
        let mut body = grounded();
        let floor = floor();
        let mut peak = 0.0_f32;
        let mut started = true;
        for frame in 0..180 {
            step(&mut body, started, frame == 0, started, 1.0 / 60.0, &floor);
            started = false;
            peak = peak.max(body.y);
            if frame > 0 && !body.jumping {
                break;
            }
        }
        assert!((peak - MIN_JUMP_HEIGHT).abs() < 2.0, "peak {peak}");
        assert_eq!(body.y, 0.0);
    }
}
