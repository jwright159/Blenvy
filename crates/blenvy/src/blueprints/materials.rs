use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::BlenvyConfig;

#[derive(Reflect, Default, Debug)]
/// struct containing the name & path of the material to apply
pub struct MaterialInfo {
    pub name: String,
    pub path: String,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// component containing the full list of `MaterialInfos` for a given entity/object
pub struct MaterialInfos(Vec<MaterialInfo>);

#[derive(Component, Default, Debug)]
pub struct MaterialProcessing {
    materials: HashMap<usize, Result<Handle<StandardMaterial>, Handle<Gltf>>>,
}
impl MaterialProcessing {
    pub fn loaded(&self) -> bool {
        self.materials.iter().all(|(_, v)| v.is_ok())
    }
}

#[derive(Component, Default, Debug)]
pub struct MaterialProcessed;

/// system that injects / replaces materials from materials library
pub(crate) fn load_material_gltfs(
    blenvy_config: Res<BlenvyConfig>,
    mut material_infos_query: Query<
        (Entity, &MaterialInfos),
        (Without<MaterialProcessed>, Without<MaterialProcessing>), // (With<BlueprintReadyForPostProcess>)
                                                                   /*(
                                                                       Added<BlueprintMaterialAssetsLoaded>,
                                                                       With<BlueprintMaterialAssetsLoaded>,
                                                                   ),*/
    >,
    asset_server: Res<AssetServer>,

    mut commands: Commands,
) {
    for (entity, material_infos) in material_infos_query.iter_mut() {
        let mut materials = HashMap::default();

        for (material_index, material_info) in material_infos.0.iter().enumerate() {
            let material_full_path = format!("{}#{}", material_info.path, material_info.name);

            if blenvy_config
                .materials_cache
                .contains_key(&material_full_path)
            {
                debug!("material is cached, retrieving");
                let material = blenvy_config
                    .materials_cache
                    .get(&material_full_path)
                    .expect("we should have the material available");
                materials.insert(material_index, Ok(material.clone()));
            } else {
                let model_handle: Handle<Gltf> = asset_server.load(material_info.path.clone());
                materials.insert(material_index, Err(model_handle));
            }
        }

        commands
            .entity(entity)
            .insert(MaterialProcessing { materials });
    }
}

pub(crate) fn inject_materials(
    mut blenvy_config: ResMut<BlenvyConfig>,
    mut material_infos_query: Query<
        (Entity, &MaterialInfos, &Children, &mut MaterialProcessing),
        Without<MaterialProcessed>,
    >,
    with_materials_and_meshes: Query<
        (),
        (
            With<Parent>,
            With<MeshMaterial3d<StandardMaterial>>,
            With<Mesh3d>,
        ),
    >,
    assets_gltf: Res<Assets<Gltf>>,
    mut commands: Commands,
) {
    for (entity, material_infos, children, mut processing) in material_infos_query.iter_mut() {
        if !processing.loaded() {
            let mut materials = processing.materials.clone();
            for (&material_index, handle) in processing.materials.iter_mut() {
                if let Err(gltf_handle) = handle {
                    if let Some(mat_gltf) = assets_gltf.get(gltf_handle) {
                        let material_info = &material_infos.0[material_index];
                        if let Some(material) =
                            mat_gltf.named_materials.get(&material_info.name as &str)
                        {
                            let material_full_path =
                                format!("{}#{}", material_info.path, material_info.name);
                            blenvy_config
                                .materials_cache
                                .insert(material_full_path, material.clone());
                            materials.insert(material_index, Ok(material.clone()));
                        }
                    }
                }
            }
            processing.materials = materials;

            if !processing.loaded() {
                continue;
            }
        }

        info!("Step 6: injecting/replacing materials");
        for (&material_index, handle) in processing.materials.iter_mut() {
            let material = handle
                .as_ref()
                .expect("we should have the material available");
            let material_info = &material_infos.0[material_index];
            for (child_index, child) in children.iter().enumerate() {
                if child_index == material_index && with_materials_and_meshes.contains(*child) {
                    info!(
                        "injecting material {}, path: {:?}",
                        material_info.name,
                        material_info.path.clone()
                    );

                    commands
                        .entity(*child)
                        .insert(MeshMaterial3d(material.clone()));
                }
            }
        }
        commands
            .entity(entity)
            .remove::<MaterialProcessing>()
            .insert(MaterialProcessed);
    }
}
