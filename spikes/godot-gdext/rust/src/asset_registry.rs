//! Runtime asset registry: load MDX models + BLP textures from a WC3 MPQ
//! at startup, cache the resulting `ArrayMesh` + `Texture2D` resources,
//! and hand them to visual nodes on demand.
//!
//! This is the integration glue for PRDs 30-35. UnitNode / GaiaNode /
//! BuildingNode / ResourceNode call `with(|reg| reg.load(...))`; the
//! existing res://*.glb path stays as a fallback if the registry isn't
//! initialised or the MPQ lookup fails.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use godot::classes::image::Format;
use godot::classes::{AnimationLibrary, Image, ImageTexture, Skeleton3D, Skin, Texture2D};
use godot::prelude::*;

use crate::blp::decode_blp;
use crate::datasource::{DataSource, MpqDataSource};
use crate::mdx::builder::{build_animation_library, build_mesh, build_skeleton, build_skin};
use crate::mdx::parser::parse_mdx;

/// Cached, ready-to-mount model. `skeleton`, `skin`, and `animations`
/// are populated only when the MDX had any bones — pure prop meshes
/// (terrain doodads, etc.) get `None` for the rigging fields.
#[derive(Clone)]
pub struct ResolvedModel {
    pub mesh: Gd<godot::classes::ArrayMesh>,
    /// One entry per mesh surface. `None` if the MDX material layer
    /// referenced no resolvable texture (replaceable team-color etc.).
    pub textures: Vec<Option<Gd<Texture2D>>>,
    pub skeleton: Option<Gd<Skeleton3D>>,
    pub skin: Option<Gd<Skin>>,
    pub animations: Option<Gd<AnimationLibrary>>,
}

pub struct AssetRegistry {
    ds: Box<dyn DataSource>,
    mdx_cache: HashMap<String, ResolvedModel>,
    tex_cache: HashMap<String, Gd<Texture2D>>,
}

impl AssetRegistry {
    pub fn open(mpq_path: &Path) -> Result<Self, String> {
        let mpq = MpqDataSource::open(mpq_path)?;
        Ok(Self {
            ds: Box::new(mpq),
            mdx_cache: HashMap::new(),
            tex_cache: HashMap::new(),
        })
    }

    /// Load (or fetch from cache) the MDX at `mdx_path` inside the MPQ.
    pub fn load(&mut self, mdx_path: &str) -> Option<ResolvedModel> {
        if let Some(cached) = self.mdx_cache.get(mdx_path) {
            return Some(cached.clone());
        }
        let bytes = self.ds.read(mdx_path)?;
        let model = match parse_mdx(&bytes) {
            Ok(m) => m,
            Err(e) => {
                godot_warn!("AssetRegistry: parse_mdx({mdx_path}) failed: {e}");
                return None;
            }
        };
        let built = build_mesh(&model);
        let mut textures = Vec::with_capacity(built.surface_textures.len());
        for (i, name) in built.surface_textures.iter().enumerate() {
            let tex = name.as_deref().and_then(|n| self.load_texture(n));
            if tex.is_none() {
                godot_print!(
                    "AssetRegistry: {mdx_path} surface {i} tex={:?} -> miss",
                    name
                );
            }
            textures.push(tex);
        }

        let (skeleton, skin, animations) = if model.bones.is_empty() && model.helpers.is_empty() {
            (None, None, None)
        } else {
            let sk = build_skeleton(&model);
            let sn = build_skin(&model, &sk);
            // Animation tracks target "Skeleton3D:BoneName" — that path
            // matches the Skeleton3D child node we mount under UnitNode.
            let lib = build_animation_library(&model, NodePath::from("Skeleton3D"));
            (Some(sk), Some(sn), Some(lib))
        };

        let resolved = ResolvedModel {
            mesh: built.mesh,
            textures,
            skeleton,
            skin,
            animations,
        };
        self.mdx_cache
            .insert(mdx_path.to_string(), resolved.clone());
        Some(resolved)
    }

    fn load_texture(&mut self, blp_path: &str) -> Option<Gd<Texture2D>> {
        if let Some(t) = self.tex_cache.get(blp_path) {
            return Some(t.clone());
        }
        // MDX paths sometimes end in .blp, sometimes have no extension.
        let candidates: Vec<String> = vec![
            blp_path.to_string(),
            if blp_path.ends_with(".blp") {
                blp_path.to_string()
            } else {
                format!("{}.blp", blp_path)
            },
            blp_path.replace(".blp", ".dds"),
        ];
        let mut bytes: Option<Vec<u8>> = None;
        for cand in &candidates {
            if let Some(b) = self.ds.read(cand) {
                bytes = Some(b);
                break;
            }
        }
        let bytes = bytes?;
        let img = decode_blp(&bytes).ok()?;
        let packed = PackedByteArray::from(img.rgba.as_slice());
        let image = Image::create_from_data(
            img.width as i32,
            img.height as i32,
            false,
            Format::RGBA8,
            &packed,
        )?;
        let tex = ImageTexture::create_from_image(&image)?;
        let tex2d: Gd<Texture2D> = tex.upcast();
        self.tex_cache.insert(blp_path.to_string(), tex2d.clone());
        Some(tex2d)
    }
}

// ── Global access ───────────────────────────────────────────────────────
//
// gdext's `Gd<T>` is !Send / !Sync because it wraps raw Godot pointers.
// Since gdext fires all Node lifecycle calls on the main thread anyway,
// thread-local storage with RefCell is the right primitive.

thread_local! {
    static REGISTRY: RefCell<Option<AssetRegistry>> = const { RefCell::new(None) };
}

/// Initialise the registry. No-op if already initialised. Returns
/// `Err` if the MPQ couldn't be opened.
pub fn init(mpq_path: &Path) -> Result<(), String> {
    let already = REGISTRY.with(|cell| cell.borrow().is_some());
    if already {
        return Ok(());
    }
    let reg = AssetRegistry::open(mpq_path)?;
    REGISTRY.with(|cell| *cell.borrow_mut() = Some(reg));
    Ok(())
}

/// Borrow the registry mutably. Returns `None` if the registry was never
/// initialised (e.g. MPQ unreachable).
pub fn with<R>(f: impl FnOnce(&mut AssetRegistry) -> R) -> Option<R> {
    REGISTRY.with(|cell| {
        let mut guard = cell.borrow_mut();
        guard.as_mut().map(f)
    })
}
