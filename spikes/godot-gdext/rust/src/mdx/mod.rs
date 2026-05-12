pub mod builder;
pub mod parser;
pub mod types;

#[cfg(test)]
mod tests {
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
}
