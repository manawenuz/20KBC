pub mod animation;
pub mod builder;
pub mod parser;
pub mod skin;
pub mod types;

use types::GeosetAlpha;

/// Sample a GEOA alpha curve at `t_ms`. Linear interp between bracketing
/// keys; clamps at curve ends; falls back to `static_alpha` only when
/// the curve has no keys at all. Matches Warsmash's runtime behavior.
pub fn sample_alpha_at(entry: &GeosetAlpha, t_ms: u32) -> f32 {
    if entry.keys.is_empty() {
        return entry.static_alpha;
    }
    let (mut prev, mut next): (Option<&(u32, f32)>, Option<&(u32, f32)>) = (None, None);
    for k in &entry.keys {
        if k.0 <= t_ms {
            prev = Some(k);
        }
        if k.0 >= t_ms && next.is_none() {
            next = Some(k);
        }
    }
    match (prev, next) {
        (Some(a), Some(b)) if a.0 == b.0 => a.1,
        (Some(a), Some(b)) => {
            let span = (b.0 - a.0) as f32;
            let f = if span > 0.0 { (t_ms - a.0) as f32 / span } else { 0.0 };
            a.1 + (b.1 - a.1) * f
        }
        (Some(a), None) => a.1,
        (None, Some(b)) => b.1,
        (None, None) => entry.static_alpha,
    }
}

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

#[cfg(test)]
mod compare_with_python {
    use super::parser::parse_mdx;
    use std::path::Path;

    #[test]
    #[cfg(target_os = "macos")]
    fn dump_peasant_from_mpq() {
        let mpq_path = Path::new("/Volumes/samGames/WC3/War3.mpq");
        if !mpq_path.exists() {
            return;
        }
        let ds = crate::datasource::MpqDataSource::open(mpq_path).unwrap();
        let bytes = crate::datasource::DataSource::read(
            &ds,
            "Units/Human/Peasant/peasant.mdx",
        )
        .unwrap();
        let model = parse_mdx(&bytes).unwrap();
        for (i, g) in model.geosets.iter().enumerate().take(2) {
            println!("Rust geoset[{}]: {} verts, {} faces", i, g.vertex_positions.len(), g.faces.len());
            println!("  first vert: {:?}", g.vertex_positions[0]);
            println!("  last vert:  {:?}", g.vertex_positions.last().unwrap());
            println!("  first face: {:?}, last face: {:?}", g.faces[0], g.faces.last().unwrap());
            let max_idx = g.faces.iter().flat_map(|(a,b,c)| [*a,*b,*c]).max().unwrap();
            println!("  max index: {} (verts count {})", max_idx, g.vertex_positions.len());
        }
    }
}

#[cfg(test)]
mod runtime_texture_check {
    use super::builder::build_mesh;
    use super::parser::parse_mdx;
    use std::path::Path;

    #[test]
    #[cfg(target_os = "macos")]
    fn peasant_surface_textures_from_mpq() {
        let mpq_path = Path::new("/Volumes/samGames/WC3/War3.mpq");
        if !mpq_path.exists() { return; }
        let ds = crate::datasource::MpqDataSource::open(mpq_path).unwrap();
        let bytes = crate::datasource::DataSource::read(&ds, "Units/Human/Peasant/peasant.mdx").unwrap();
        let model = parse_mdx(&bytes).unwrap();
        println!("Peasant TEXS chunk:");
        for (i, t) in model.textures.iter().enumerate() {
            println!("  texture[{}]: replaceable_id={} file_name={:?}", i, t.replaceable_id, t.file_name);
        }
        println!("Peasant MTLS chunk:");
        for (i, m) in model.materials.iter().enumerate() {
            for (j, l) in m.layers.iter().enumerate() {
                println!("  material[{}].layer[{}]: texture_id={} filter_mode={}", i, j, l.texture_id, l.filter_mode);
            }
        }
        let built = build_mesh(&model);
        println!("Resolved surface_textures:");
        for (i, t) in built.surface_textures.iter().enumerate() {
            println!("  surface[{}]: {:?}", i, t);
        }
    }
}

#[cfg(test)]
mod tex_path_test {
    use std::path::Path;
    #[test]
    #[cfg(target_os = "macos")]
    fn check_blp_path_resolution() {
        let mpq_path = Path::new("/Volumes/samGames/WC3/War3.mpq");
        if !mpq_path.exists() { return; }
        let ds = crate::datasource::MpqDataSource::open(mpq_path).unwrap();
        // The path that comes out of MDX TEXS chunk
        for p in [r"Textures\Peasant.blp", "Textures/Peasant.blp", "textures/peasant.blp"] {
            let r = crate::datasource::DataSource::read(&ds, p);
            println!("read({:?}) -> {} bytes", p, r.as_ref().map(|v| v.len()).unwrap_or(0));
        }
    }
}
