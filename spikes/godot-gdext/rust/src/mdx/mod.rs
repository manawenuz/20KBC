pub mod animation;
pub mod builder;
pub mod parser;
pub mod skin;
pub mod types;

#[cfg(test)]
mod tests {
    use godot::prelude::NodePath;

    #[test]
    fn parse_peasant() {
        let bytes = std::fs::read("tests/data/peasant.mdx").unwrap();
        let model = super::parser::parse_mdx(&bytes).unwrap();
        assert!(model.geosets.len() >= 6);
        assert!(!model.bones.is_empty());
        assert!(model.sequences.iter().any(|s| s.name.contains("Stand")));
    }

    #[test]
    fn build_peasant_mesh_kept_2_surfaces() {
        let bytes = std::fs::read("tests/data/peasant.mdx").unwrap();
        let model = super::parser::parse_mdx(&bytes).unwrap();
        let built = super::builder::build_mesh(&model);
        // Strict filter keeps geosets 0 and 1 (head + body) = 2 surfaces
        assert_eq!(built.surface_textures.len(), 2);
    }

    #[test]
    fn peasant_skeleton_22_bones() {
        let bytes = std::fs::read("tests/data/peasant.mdx").unwrap();
        let model = super::parser::parse_mdx(&bytes).unwrap();
        let skel = super::builder::build_skeleton(&model);
        assert!(skel.get_bone_count() >= 22); // bones + helpers
    }

    #[test]
    fn peasant_animation_library_has_stand_walk() {
        let bytes = std::fs::read("tests/data/peasant.mdx").unwrap();
        let model = super::parser::parse_mdx(&bytes).unwrap();
        let lib = super::builder::build_animation_library(&model, NodePath::from("Skeleton3D"));
        let names = lib.get_animation_list();
        let name_vec: Vec<String> = names.iter_shared().map(|n| n.to_string()).collect();
        assert!(name_vec.iter().any(|n| n == "Stand"));
        assert!(name_vec.iter().any(|n| n == "Walk"));
    }
}
