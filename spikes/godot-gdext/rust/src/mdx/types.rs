#[derive(Debug, Clone)]
pub struct TextureRef {
    pub replaceable_id: u32,
    pub file_name: String,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct LayerDef {
    pub filter_mode: u32,
    pub shading_flags: u32,
    pub texture_id: u32,
    pub texture_animation_id: u32,
    pub coord_id: u32,
    pub alpha: f32,
}

#[derive(Debug, Clone)]
pub struct MaterialDef {
    pub priority_plane: u32,
    pub flags: u32,
    pub layers: Vec<LayerDef>,
}

#[derive(Debug, Clone)]
pub struct Sequence {
    pub name: String,
    pub start_ms: u32,
    pub end_ms: u32,
    pub move_speed: f32,
    pub loop_: bool,
}

#[derive(Debug, Clone)]
pub struct Bone {
    pub name: String,
    pub object_id: u32,
    pub parent_id: u32,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct Geoset {
    pub vertex_positions: Vec<[f32; 3]>,
    pub vertex_normals: Vec<[f32; 3]>,
    pub faces: Vec<(u16, u16, u16)>,
    pub uvs: Vec<[f32; 2]>,
    pub material_id: u32,
    pub selection_group: u32,
    pub selection_flags: u32,
    pub vertex_groups: Vec<u8>,
    pub matrix_group_counts: Vec<u32>,
    pub matrix_indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct GeosetAlpha {
    pub static_alpha: f32,
    pub keys: Vec<(u32, f32)>,
}

#[derive(Debug, Clone)]
pub struct MdxModel {
    pub textures: Vec<TextureRef>,
    pub materials: Vec<MaterialDef>,
    pub geosets: Vec<Geoset>,
    pub bones: Vec<Bone>,
    pub helpers: Vec<Bone>,
    pub pivots: Vec<[f32; 3]>,
    pub sequences: Vec<Sequence>,
    pub geoset_alpha: Vec<Option<GeosetAlpha>>,
}
