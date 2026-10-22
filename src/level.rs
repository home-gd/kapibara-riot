use bevy::prelude::*;

const PAD_COLUMNS: u32 = 12;
const PAD_TILE: f32 = 32.0;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb_u8(8, 166, 213)))
            .add_systems(Startup, spawn_level);
    }
}

/// One walkable top from the Tiled object layer.
///
/// `one_way` platforms can be landed on and dropped through. The ground is not one-way.
#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub one_way: bool,
}

#[derive(Resource)]
pub struct Level {
    pub surfaces: Vec<Surface>,
    pub bounds: Rect,
    pub spawn: Vec2,
}

#[derive(Clone, Copy, Debug)]
struct PadTile {
    center: Vec2,
    col: u32,
    row: u32,
}

struct ParsedMap {
    surfaces: Vec<Surface>,
    tiles: Vec<PadTile>,
    bounds: Rect,
    spawn: Vec2,
}

pub fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    let parsed = parse_map(&read_map());
    let pad = asset_server.load("level1/Pad_1_3.png");

    commands.spawn((
        Camera2d,
        Transform::from_xyz(parsed.spawn.x, parsed.spawn.y, 999.0),
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("level1/origbig.png")),
        Transform::default(),
    ));

    for tile in &parsed.tiles {
        commands.spawn((
            Sprite {
                image: pad.clone(),
                rect: Some(Rect::new(
                    tile.col as f32 * PAD_TILE,
                    tile.row as f32 * PAD_TILE,
                    (tile.col + 1) as f32 * PAD_TILE,
                    (tile.row + 1) as f32 * PAD_TILE,
                )),
                ..default()
            },
            Transform::from_xyz(tile.center.x, tile.center.y, 1.0),
        ));
    }

    commands.insert_resource(Level {
        surfaces: parsed.surfaces,
        bounds: parsed.bounds,
        spawn: parsed.spawn,
    });
}

fn read_map() -> String {
    let relative = "assets/level1/map.tmx";
    if let Ok(text) = std::fs::read_to_string(relative) {
        return text;
    }
    let bundled = format!("{}/{relative}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&bundled)
        .unwrap_or_else(|_| panic!("could not read {relative} (also tried {bundled})"))
}

/// Tiled's origin is the top-left and Y grows downward. World Y grows upward,
/// and the background image is centered on the origin.
fn tiled_to_world(px: f32, py: f32, bg_w: f32, bg_h: f32) -> Vec2 {
    Vec2::new(px - bg_w * 0.5, bg_h * 0.5 - py)
}

fn parse_map(text: &str) -> ParsedMap {
    let map_tag = element_open(text, "map").expect("map element");
    let tile_w = attr_f32(map_tag, "tilewidth").expect("tilewidth");
    let tile_h = attr_f32(map_tag, "tileheight").expect("tileheight");
    let image = element_open(text, "image").expect("background image");
    let bg_w = attr_f32(image, "width").expect("image width");
    let bg_h = attr_f32(image, "height").expect("image height");
    let pad_gid = tileset_first_gid(text, "Pad_1_3");

    let mut surfaces: Vec<Surface> = object_boxes(text)
        .into_iter()
        .map(|object| {
            let left = tiled_to_world(object.x, 0.0, bg_w, bg_h).x;
            let top = tiled_to_world(0.0, object.y, bg_w, bg_h).y;
            Surface {
                left,
                right: left + object.width,
                top,
                one_way: true,
            }
        })
        .collect();

    let ground_top = surfaces
        .iter()
        .map(|surface| surface.top)
        .fold(f32::INFINITY, f32::min);
    for surface in &mut surfaces {
        // The lowest authored box is the floor of the painting. It cannot be dropped through.
        if (surface.top - ground_top).abs() <= 1.0 {
            surface.one_way = false;
        }
    }

    let ground = surfaces
        .iter()
        .find(|surface| !surface.one_way)
        .expect("level1 has no ground collision");
    let spawn = Vec2::new(ground.left + 220.0, ground.top);

    let tiles = pad_tiles(text, pad_gid, tile_w, tile_h, bg_w, bg_h);

    ParsedMap {
        surfaces,
        tiles,
        bounds: Rect::new(-bg_w * 0.5, -bg_h * 0.5, bg_w * 0.5, bg_h * 0.5),
        spawn,
    }
}

struct ObjectBox {
    x: f32,
    y: f32,
    width: f32,
}

fn object_boxes(text: &str) -> Vec<ObjectBox> {
    element_opens(text, "object")
        .filter_map(|tag| {
            let _height = attr_f32(tag, "height")?;
            Some(ObjectBox {
                x: attr_f32(tag, "x")?,
                y: attr_f32(tag, "y")?,
                width: attr_f32(tag, "width")?,
            })
        })
        .collect()
}

fn pad_tiles(
    text: &str,
    first_gid: u32,
    tile_w: f32,
    tile_h: f32,
    bg_w: f32,
    bg_h: f32,
) -> Vec<PadTile> {
    let mut tiles = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("<chunk ") {
        rest = &rest[start..];
        let tag_end = rest.find('>').expect("chunk tag");
        let tag = &rest[..tag_end];
        let origin_x = attr_f32(tag, "x").expect("chunk x") as i32;
        let origin_y = attr_f32(tag, "y").expect("chunk y") as i32;
        let width = attr_f32(tag, "width").expect("chunk width") as i32;
        let body_start = tag_end + 1;
        let body_end = rest[body_start..].find("</chunk>").expect("chunk end");
        let csv = &rest[body_start..body_start + body_end];
        let mut index = 0;
        for token in csv.split(|c: char| c == ',' || c.is_whitespace()) {
            if token.is_empty() {
                continue;
            }
            let gid = token.parse::<u32>().unwrap_or(0) & 0x1FFF_FFFF;
            if gid >= first_gid {
                let local = gid - first_gid;
                let col = local % PAD_COLUMNS;
                let row = local / PAD_COLUMNS;
                let tile_x = origin_x + (index % width);
                let tile_y = origin_y + (index / width);
                let center = tiled_to_world(
                    (tile_x as f32 + 0.5) * tile_w,
                    (tile_y as f32 + 0.5) * tile_h,
                    bg_w,
                    bg_h,
                );
                tiles.push(PadTile { center, col, row });
            }
            index += 1;
        }
        rest = &rest[body_start + body_end..];
    }
    tiles
}

fn tileset_first_gid(text: &str, source_needle: &str) -> u32 {
    element_opens(text, "tileset")
        .find(|tag| attr_str(tag, "source").is_some_and(|source| source.contains(source_needle)))
        .and_then(|tag| attr_f32(tag, "firstgid"))
        .expect("Pad tileset") as u32
}

fn element_open<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    element_opens(text, name).next()
}

fn element_opens<'a>(text: &'a str, name: &str) -> impl Iterator<Item = &'a str> {
    let open = format!("<{name} ");
    let alt = format!("<{name}>");
    let mut rest = text;
    std::iter::from_fn(move || {
        let start = rest.find(&open).or_else(|| rest.find(&alt))?;
        rest = &rest[start..];
        let end = rest.find('>')?;
        let tag = &rest[..end];
        rest = &rest[end + 1..];
        Some(tag)
    })
}

fn attr_f32(tag: &str, name: &str) -> Option<f32> {
    attr_str(tag, name)?.parse().ok()
}

fn attr_str<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let value = tag[start..].split('"').next()?;
    Some(value)
}
