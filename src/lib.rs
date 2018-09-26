extern crate amethyst;

use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::{Parent, Transform};
use amethyst::ecs::*;
use amethyst::prelude::*;
use amethyst::renderer::{
    Material, MaterialDefaults, Mesh, MeshData, MeshHandle, PosNormTangTex, Texture, TextureData,
    TextureHandle, TextureMetadata,
};
use amethyst::xr::components::TrackingDevice;
use amethyst::xr::{TrackerModelLoadStatus, XRInfo};

pub type ComponentModel = (String, MeshHandle, TextureHandle);

pub struct XRTrackerModels;

impl<'a> System<'a> for XRTrackerModels {
    type SystemData = (
        WriteExpect<'a, XRInfo>,
        Entities<'a>,
        Read<'a, LazyUpdate>,
        WriteStorage<'a, TrackingDevice>,
        ReadStorage<'a, XRModelEnabled>,
        ReadStorage<'a, XRModelInfo>,
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<Mesh>>,
        Read<'a, AssetStorage<Texture>>,
        ReadExpect<'a, MaterialDefaults>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut info,
            entities,
            updater,
            mut trackers,
            enabled,
            model_infos,
            loader,
            meshes,
            textures,
            material_defaults,
        ) = data;

        for (entity, tracker, _, _) in (&*entities, &trackers, &enabled, !&model_infos).join() {
            if let TrackerModelLoadStatus::Available(models) =
                info.backend().get_tracker_models(tracker.id())
            {
                let mut component_models: Vec<_> = Vec::new();

                for (i, model_info) in models.into_iter().enumerate() {
                    let name = model_info
                        .component_name
                        .unwrap_or_else(|| String::from("unkown"))
                        + &format!("-{}-{}", tracker.id(), i);

                    let vertices: Vec<_> = model_info
                        .vertices
                        .iter()
                        .map(|v| PosNormTangTex {
                            position: v.position,
                            normal: v.normal,
                            tangent: v.tangent,
                            tex_coord: v.tex_coord,
                        }).collect();
                    let vertices = MeshData::PosNormTangTex(
                        model_info
                            .indices
                            .iter()
                            .map(|i| vertices[*i as usize].clone())
                            .collect(),
                    );
                    let mesh = loader.load_from_data(vertices, (), &meshes);

                    let texture = if let Some(texture) = model_info.texture {
                        let texture_data = TextureData::U8(
                            texture.data,
                            TextureMetadata::default().with_size(texture.size.0, texture.size.1),
                        );
                        let texture = loader.load_from_data(texture_data, (), &textures);
                        texture
                    } else {
                        loader.load_from_data([1.0; 4].into(), (), &textures)
                    };

                    let material = Material {
                        albedo: texture.clone(),
                        ..material_defaults.0.clone()
                    };

                    updater
                        .create_entity(&*entities)
                        .named(name.clone())
                        .with(Parent { entity })
                        .with(Transform::default())
                        .with(mesh.clone())
                        .with(material)
                        .build();

                    component_models.push((name, mesh, texture));
                }

                updater.insert(entity, XRModelInfo { component_models });
            }
        }
    }
}

#[derive(Default)]
pub struct XRModelEnabled;

impl Component for XRModelEnabled {
    type Storage = NullStorage<XRModelEnabled>;
}

pub struct XRModelInfo {
    pub(crate) component_models: Vec<ComponentModel>,
}

impl Component for XRModelInfo {
    type Storage = HashMapStorage<XRModelInfo>;
}
