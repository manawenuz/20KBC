use super::types::*;

pub fn parse_mdx(bytes: &[u8]) -> Result<MdxModel, String> {
    if bytes.len() < 4 || &bytes[0..4] != b"MDLX" {
        return Err("Not a valid MDX file (missing MDLX magic)".into());
    }

    let mut _version = None;
    let mut textures = Vec::new();
    let mut materials = Vec::new();
    let mut geosets = Vec::new();
    let mut bones = Vec::new();
    let mut helpers = Vec::new();
    let mut pivots = Vec::new();
    let mut sequences = Vec::new();
    let mut geoset_alpha_map = std::collections::HashMap::<usize, GeosetAlpha>::new();

    let mut offset = 4usize;
    while offset + 8 <= bytes.len() {
        let chunk_id = std::str::from_utf8(&bytes[offset..offset + 4])
            .map_err(|e| format!("invalid chunk tag: {e}"))?;
        let chunk_size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        if offset + 8 + chunk_size > bytes.len() {
            break;
        }
        let chunk_data = &bytes[offset + 8..offset + 8 + chunk_size];

        match chunk_id {
            "VERS" => {
                if chunk_data.len() >= 4 {
                    _version = Some(u32::from_le_bytes(chunk_data[0..4].try_into().unwrap()));
                }
            }
            "TEXS" => {
                let mut tex_offset = 0usize;
                while tex_offset + 268 <= chunk_data.len() {
                    let replaceable_id = u32::from_le_bytes(chunk_data[tex_offset..tex_offset + 4].try_into().unwrap());
                    let file_name = read_c_str(&chunk_data[tex_offset + 4..tex_offset + 264]);
                    let flags = u32::from_le_bytes(chunk_data[tex_offset + 264..tex_offset + 268].try_into().unwrap());
                    textures.push(TextureRef { replaceable_id, file_name, flags });
                    tex_offset += 268;
                }
            }
            "MTLS" => {
                let mut mtl_offset = 0usize;
                let mtl_end = chunk_data.len();
                while mtl_offset + 4 <= mtl_end {
                    let inclusive_size = u32::from_le_bytes(chunk_data[mtl_offset..mtl_offset + 4].try_into().unwrap()) as usize;
                    if mtl_offset + inclusive_size > mtl_end {
                        break;
                    }
                    let mtl_data = &chunk_data[mtl_offset..mtl_offset + inclusive_size];
                    let mut sub_off = 4usize;
                    let priority_plane = u32::from_le_bytes(mtl_data[sub_off..sub_off + 4].try_into().unwrap());
                    let flags = u32::from_le_bytes(mtl_data[sub_off + 4..sub_off + 8].try_into().unwrap());
                    sub_off += 8;
                    let mut layers = Vec::new();
                    if sub_off + 8 <= mtl_data.len() && &mtl_data[sub_off..sub_off + 4] == b"LAYS" {
                        let layers_count = u32::from_le_bytes(mtl_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                        sub_off += 8;
                        for _ in 0..layers_count {
                            if sub_off + 4 > mtl_data.len() {
                                break;
                            }
                            let layer_inclusive = u32::from_le_bytes(mtl_data[sub_off..sub_off + 4].try_into().unwrap()) as usize;
                            if sub_off + layer_inclusive > mtl_data.len() {
                                break;
                            }
                            let filter_mode = u32::from_le_bytes(mtl_data[sub_off + 4..sub_off + 8].try_into().unwrap());
                            let shading_flags = u32::from_le_bytes(mtl_data[sub_off + 8..sub_off + 12].try_into().unwrap());
                            let texture_id = u32::from_le_bytes(mtl_data[sub_off + 12..sub_off + 16].try_into().unwrap());
                            let tex_anim_id = u32::from_le_bytes(mtl_data[sub_off + 16..sub_off + 20].try_into().unwrap());
                            let coord_id = u32::from_le_bytes(mtl_data[sub_off + 20..sub_off + 24].try_into().unwrap());
                            let alpha = f32::from_le_bytes(mtl_data[sub_off + 24..sub_off + 28].try_into().unwrap());
                            layers.push(LayerDef {
                                filter_mode,
                                shading_flags,
                                texture_id,
                                texture_animation_id: tex_anim_id,
                                coord_id,
                                alpha,
                            });
                            sub_off += layer_inclusive;
                        }
                    }
                    materials.push(MaterialDef { priority_plane, flags, layers });
                    mtl_offset += inclusive_size;
                }
            }
            "GEOS" => {
                let mut geos_offset = 0usize;
                let geos_end = chunk_data.len();
                while geos_offset + 4 <= geos_end {
                    let inclusive_size = u32::from_le_bytes(chunk_data[geos_offset..geos_offset + 4].try_into().unwrap()) as usize;
                    if geos_offset + inclusive_size > geos_end {
                        break;
                    }
                    let geoset_data = &chunk_data[geos_offset..geos_offset + inclusive_size];
                    let mut g = Geoset {
                        vertex_positions: Vec::new(),
                        vertex_normals: Vec::new(),
                        faces: Vec::new(),
                        uvs: Vec::new(),
                        material_id: 0,
                        selection_group: 0,
                        selection_flags: 0,
                        vertex_groups: Vec::new(),
                        matrix_group_counts: Vec::new(),
                        matrix_indices: Vec::new(),
                    };
                    let mut sub_off = 4usize;
                    let g_end = geoset_data.len();
                    while sub_off < g_end {
                        if sub_off + 4 > g_end {
                            break;
                        }
                        let tag_bytes = &geoset_data[sub_off..sub_off + 4];
                        let is_tag = tag_bytes.iter().all(|&b| b.is_ascii_alphabetic() || b.is_ascii_whitespace() || b == b'_');
                        if is_tag {
                            let sub_id = std::str::from_utf8(tag_bytes).map_err(|e| format!("invalid geoset sub-tag: {e}"))?;
                            match sub_id {
                                "VRTX" => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    for i in 0..count {
                                        g.vertex_positions.push(read_float3(geoset_data, sub_off + 8 + i * 12));
                                    }
                                    sub_off += 8 + count * 12;
                                }
                                "NRMS" => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    for i in 0..count {
                                        g.vertex_normals.push(read_float3(geoset_data, sub_off + 8 + i * 12));
                                    }
                                    sub_off += 8 + count * 12;
                                }
                                "PTYP" => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    sub_off += 8 + count * 4;
                                }
                                "PCNT" => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    sub_off += 8 + count * 4;
                                }
                                "PVTX" => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    for i in (0..count).step_by(3) {
                                        let i0 = u16::from_le_bytes(geoset_data[sub_off + 8 + i * 2..sub_off + 10 + i * 2].try_into().unwrap());
                                        let i1 = u16::from_le_bytes(geoset_data[sub_off + 10 + i * 2..sub_off + 12 + i * 2].try_into().unwrap());
                                        let i2 = u16::from_le_bytes(geoset_data[sub_off + 12 + i * 2..sub_off + 14 + i * 2].try_into().unwrap());
                                        g.faces.push((i0, i1, i2));
                                    }
                                    sub_off += 8 + count * 2;
                                }
                                "GNDX" => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    g.vertex_groups.extend_from_slice(&geoset_data[sub_off + 8..sub_off + 8 + count]);
                                    sub_off += 8 + count;
                                }
                                "MTGC" => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    for i in 0..count {
                                        g.matrix_group_counts.push(u32::from_le_bytes(
                                            geoset_data[sub_off + 8 + i * 4..sub_off + 12 + i * 4].try_into().unwrap(),
                                        ));
                                    }
                                    sub_off += 8 + count * 4;
                                }
                                "MATS" => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    for i in 0..count {
                                        g.matrix_indices.push(u32::from_le_bytes(
                                            geoset_data[sub_off + 8 + i * 4..sub_off + 12 + i * 4].try_into().unwrap(),
                                        ));
                                    }
                                    sub_off += 8 + count * 4;
                                }
                                "UVAS" => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    sub_off += 8;
                                    for _ in 0..count {
                                        if sub_off + 8 > geoset_data.len() {
                                            break;
                                        }
                                        let uv_tag = std::str::from_utf8(&geoset_data[sub_off..sub_off + 4]).unwrap_or("");
                                        let uv_count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                        if uv_tag == "UVBS" {
                                            for i in 0..uv_count {
                                                g.uvs.push(read_float2(geoset_data, sub_off + 8 + i * 8));
                                            }
                                        }
                                        sub_off += 8 + uv_count * 8;
                                    }
                                }
                                "TANG" | "SKIN" => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    let skip = if sub_id == "TANG" { count * 4 } else { count };
                                    sub_off += 8 + skip;
                                }
                                _ => {
                                    let count = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap()) as usize;
                                    sub_off += 8 + count * 4;
                                }
                            }
                        } else {
                            // Trailing fixed fields
                            if sub_off + 12 <= g_end {
                                g.material_id = u32::from_le_bytes(geoset_data[sub_off..sub_off + 4].try_into().unwrap());
                                g.selection_group = u32::from_le_bytes(geoset_data[sub_off + 4..sub_off + 8].try_into().unwrap());
                                g.selection_flags = u32::from_le_bytes(geoset_data[sub_off + 8..sub_off + 12].try_into().unwrap());
                            }
                            sub_off += 40; // bounds_radius + min_extent + max_extent
                            if sub_off + 4 <= g_end {
                                let extents_count = u32::from_le_bytes(geoset_data[sub_off..sub_off + 4].try_into().unwrap()) as usize;
                                sub_off += 4 + extents_count * 28;
                            }
                        }
                    }
                    geosets.push(g);
                    geos_offset += inclusive_size;
                }
            }
            "BONE" => {
                bones = parse_bone_nodes(chunk_data)?;
            }
            "HELP" => {
                helpers = parse_bone_nodes(chunk_data)?;
            }
            "GEOA" => {
                geoset_alpha_map = parse_geoset_animations(chunk_data)?;
            }
            "PIVT" => {
                let count = chunk_data.len() / 12;
                for i in 0..count {
                    pivots.push(read_float3(chunk_data, i * 12));
                }
            }
            "SEQS" => {
                let mut seq_offset = 0usize;
                while seq_offset + 132 <= chunk_data.len() {
                    let name = read_c_str(&chunk_data[seq_offset..seq_offset + 80]);
                    let start_ms = u32::from_le_bytes(chunk_data[seq_offset + 80..seq_offset + 84].try_into().unwrap());
                    let end_ms = u32::from_le_bytes(chunk_data[seq_offset + 84..seq_offset + 88].try_into().unwrap());
                    let move_speed = f32::from_le_bytes(chunk_data[seq_offset + 88..seq_offset + 92].try_into().unwrap());
                    let no_loop = u32::from_le_bytes(chunk_data[seq_offset + 92..seq_offset + 96].try_into().unwrap());
                    sequences.push(Sequence {
                        name,
                        start_ms,
                        end_ms,
                        move_speed,
                        loop_: no_loop == 0,
                    });
                    seq_offset += 132;
                }
            }
            _ => {}
        }

        offset += 8 + chunk_size;
    }

    // Build geoset_alpha Vec<Option<GeosetAlpha>> indexed by geoset id
    let max_geo_id = geosets.len().max(geoset_alpha_map.keys().copied().max().unwrap_or(0) + 1);
    let mut geoset_alpha = vec![None; max_geo_id];
    for (id, alpha) in geoset_alpha_map {
        if id < geoset_alpha.len() {
            geoset_alpha[id] = Some(alpha);
        }
    }

    Ok(MdxModel {
        textures,
        materials,
        geosets,
        bones,
        helpers,
        pivots,
        sequences,
        geoset_alpha,
    })
}

fn read_float3(data: &[u8], offset: usize) -> [f32; 3] {
    [
        f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()),
        f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()),
        f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()),
    ]
}

fn read_float2(data: &[u8], offset: usize) -> [f32; 2] {
    [
        f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()),
        f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()),
    ]
}

fn read_c_str(data: &[u8]) -> String {
    data.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect::<String>()
        .replace("\u{FFFD}", "")
}

fn parse_bone_nodes(chunk_data: &[u8]) -> Result<Vec<Bone>, String> {
    let mut bones = Vec::new();
    let mut i = 0usize;
    while i + 96 <= chunk_data.len() {
        let size = u32::from_le_bytes(chunk_data[i..i + 4].try_into().unwrap()) as usize;
        if size < 96 || i + size > chunk_data.len() {
            i += 4;
            continue;
        }
        let name = read_c_str(&chunk_data[i + 4..i + 84]);
        let obj_id = u32::from_le_bytes(chunk_data[i + 84..i + 88].try_into().unwrap());
        let parent = u32::from_le_bytes(chunk_data[i + 88..i + 92].try_into().unwrap());
        let flags = u32::from_le_bytes(chunk_data[i + 92..i + 96].try_into().unwrap());
        let mut translations = Vec::new();
        let mut rotations = Vec::new();
        let mut scales = Vec::new();

        let mut track_off = 96usize;
        while track_off + 16 <= size {
            let tag = std::str::from_utf8(&chunk_data[i + track_off..i + track_off + 4]).unwrap_or("");
            let count = u32::from_le_bytes(chunk_data[i + track_off + 4..i + track_off + 8].try_into().unwrap()) as usize;
            let interp = u32::from_le_bytes(chunk_data[i + track_off + 8..i + track_off + 12].try_into().unwrap());
            // Skip global sequence id
            let data_start = i + track_off + 16;
            match tag {
                "KGTR" => {
                    let kf_size = if interp <= 1 { 16 } else { 40 };
                    let mut data_off = data_start;
                    for _ in 0..count {
                        if data_off + kf_size > chunk_data.len() {
                            break;
                        }
                        let t_ms = u32::from_le_bytes(chunk_data[data_off..data_off + 4].try_into().unwrap());
                        let value = read_float3(chunk_data, data_off + 4);
                        translations.push(TranslationKf { time_ms: t_ms, value, interpolation: interp });
                        data_off += kf_size;
                    }
                    track_off += 16 + count * kf_size;
                }
                "KGRT" => {
                    let kf_size = if interp <= 1 { 20 } else { 52 };
                    let mut data_off = data_start;
                    for _ in 0..count {
                        if data_off + kf_size > chunk_data.len() {
                            break;
                        }
                        let t_ms = u32::from_le_bytes(chunk_data[data_off..data_off + 4].try_into().unwrap());
                        let value = [
                            f32::from_le_bytes(chunk_data[data_off + 4..data_off + 8].try_into().unwrap()),
                            f32::from_le_bytes(chunk_data[data_off + 8..data_off + 12].try_into().unwrap()),
                            f32::from_le_bytes(chunk_data[data_off + 12..data_off + 16].try_into().unwrap()),
                            f32::from_le_bytes(chunk_data[data_off + 16..data_off + 20].try_into().unwrap()),
                        ];
                        rotations.push(RotationKf { time_ms: t_ms, value, interpolation: interp });
                        data_off += kf_size;
                    }
                    track_off += 16 + count * kf_size;
                }
                "KGSC" => {
                    let kf_size = if interp <= 1 { 16 } else { 40 };
                    let mut data_off = data_start;
                    for _ in 0..count {
                        if data_off + kf_size > chunk_data.len() {
                            break;
                        }
                        let t_ms = u32::from_le_bytes(chunk_data[data_off..data_off + 4].try_into().unwrap());
                        let value = read_float3(chunk_data, data_off + 4);
                        scales.push(ScaleKf { time_ms: t_ms, value, interpolation: interp });
                        data_off += kf_size;
                    }
                    track_off += 16 + count * kf_size;
                }
                _ => break,
            }
        }

        if obj_id < 500 && (parent == 0xFFFFFFFF || parent < 500) {
            bones.push(Bone {
                name,
                object_id: obj_id,
                parent_id: parent,
                flags,
                translations,
                rotations,
                scales,
            });
            i += size;
            continue;
        }
        i += 4;
    }
    Ok(bones)
}

fn parse_geoset_animations(chunk_data: &[u8]) -> Result<std::collections::HashMap<usize, GeosetAlpha>, String> {
    let mut map = std::collections::HashMap::new();
    let mut i = 0usize;
    while i + 28 <= chunk_data.len() {
        let size = u32::from_le_bytes(chunk_data[i..i + 4].try_into().unwrap()) as usize;
        if size < 28 || i + size > chunk_data.len() {
            break;
        }
        let static_alpha = f32::from_le_bytes(chunk_data[i + 4..i + 8].try_into().unwrap());
        let geoset_id = u32::from_le_bytes(chunk_data[i + 24..i + 28].try_into().unwrap()) as usize;
        let mut keys = Vec::new();

        let mut track_off = 28usize;
        while track_off + 16 <= size {
            let tag = std::str::from_utf8(&chunk_data[i + track_off..i + track_off + 4]).unwrap_or("");
            let count = u32::from_le_bytes(chunk_data[i + track_off + 4..i + track_off + 8].try_into().unwrap()) as usize;
            let interp = u32::from_le_bytes(chunk_data[i + track_off + 8..i + track_off + 12].try_into().unwrap());
            match tag {
                "KGAO" => {
                    let kf_size = if interp <= 1 { 8 } else { 16 };
                    let mut data_off = i + track_off + 16;
                    for _ in 0..count {
                        if data_off + kf_size > chunk_data.len() {
                            break;
                        }
                        let t_ms = u32::from_le_bytes(chunk_data[data_off..data_off + 4].try_into().unwrap());
                        let a = f32::from_le_bytes(chunk_data[data_off + 4..data_off + 8].try_into().unwrap());
                        keys.push((t_ms, a));
                        data_off += kf_size;
                    }
                    track_off += 16 + count * kf_size;
                }
                "KGAC" => {
                    let kf_size = if interp <= 1 { 16 } else { 40 };
                    track_off += 16 + count * kf_size;
                }
                _ => break,
            }
        }

        map.insert(geoset_id, GeosetAlpha { static_alpha, keys });
        i += size;
    }
    Ok(map)
}
