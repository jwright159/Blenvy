pub mod utils;
use utils::capitalize_first_letter;

pub mod ronstring_to_reflect_component;

pub mod process_gltfs;
use process_gltfs::add_components_from_gltf_extras;

pub mod blender_settings;

use bevy::{
    ecs::{component::Component, reflect::ReflectComponent},
    prelude::{App, IntoSystemConfigs, Plugin, SystemSet, Update},
    reflect::Reflect,
};

/// A Bevy plugin for extracting components from gltf files and automatically adding them to the relevant entities
/// It will automatically run every time you load a gltf file
/// Add this plugin to your Bevy app to get access to this feature
/// ```
/// # use bevy::prelude::*;
/// # use bevy::gltf::*;
/// # use blenvy::ComponentsFromGltfPlugin;
///
/// //too barebones of an example to be meaningfull, please see https://github.com/kaosat-dev/Blenvy/examples/basic for a real example
/// fn main() {
///    App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(ComponentsFromGltfPlugin)
///         .add_systems(Startup, spawn_level)
///         .run();
/// }
///
/// fn spawn_level(
///   asset_server: Res<AssetServer>,
///   mut commands: Commands,
///   keycode: Res<ButtonInput<KeyCode>>,
/// ){
/// if keycode.just_pressed(KeyCode::Enter) {
///  commands.spawn((
///   SceneRoot(asset_server.load("basic/models/level1.glb")),
///   Transform::from_xyz(2.0, 0.0, -5.0),
///  ));
/// }
///}
/// ```

/// this is a flag component to tag a processed gltf, to avoid processing things multiple times
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct GltfProcessed;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
/// systemset to order your systems after the component injection when needed
pub enum GltfComponentsSet {
    Injection,
}

#[derive(Default)]
pub struct ComponentsFromGltfPlugin;

impl Plugin for ComponentsFromGltfPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(blender_settings::plugin)
            .register_type::<GltfProcessed>()
            .add_systems(
                Update,
                (add_components_from_gltf_extras).in_set(GltfComponentsSet::Injection),
            );
    }
}
