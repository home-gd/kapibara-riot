mod level;
mod player;

use bevy::prelude::*;
use level::LevelPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((LevelPlugin, PlayerPlugin))
        .run();
}
