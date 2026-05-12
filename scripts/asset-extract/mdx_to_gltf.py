#!/usr/bin/env python3
"""
Minimal WC3 MDX → glTF 2.0 converter.
Parses MDX version 800 static mesh + UVs + first texture layer (no skinning, no animation).
"""

import argparse
import ctypes
import json
import math
import os
import struct
import sys
from io import BytesIO
from pathlib import Path

try:
    from PIL import Image
    _HAS_PILLOW = True
except ImportError:
    _HAS_PILLOW = False

# ---------------------------------------------------------------------------
# StormLib bindings (shared with extract.py)
# ---------------------------------------------------------------------------

_LIB_PATHS = [
    "/opt/homebrew/lib/libstorm.dylib",
    "/usr/local/lib/libstorm.dylib",
    "libstorm.dylib",
    "libstorm.so",
]

_storm = None
for _lp in _LIB_PATHS:
    try:
        _storm = ctypes.CDLL(_lp)
        break
    except OSError:
        continue

if _storm is None:
    raise RuntimeError("StormLib not found. Install it (e.g. 'brew install stormlib').")


class _SFILE_FIND_DATA(ctypes.Structure):
    _fields_ = [
        ("cFileName", ctypes.c_char * 1024),
        ("szPlainName", ctypes.c_char_p),
        ("dwHashIndex", ctypes.c_uint32),
        ("dwBlockIndex", ctypes.c_uint32),
        ("dwFileSize", ctypes.c_uint32),
        ("dwFileFlags", ctypes.c_uint32),
        ("dwCompSize", ctypes.c_uint32),
        ("dwFileTimeLo", ctypes.c_uint32),
        ("dwFileTimeHi", ctypes.c_uint32),
        ("lcLocale", ctypes.c_uint32),
    ]


_storm.SFileOpenArchive.argtypes = [
    ctypes.c_char_p,
    ctypes.c_uint32,
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_void_p),
]
_storm.SFileOpenArchive.restype = ctypes.c_bool

_storm.SFileCloseArchive.argtypes = [ctypes.c_void_p]
_storm.SFileCloseArchive.restype = ctypes.c_bool

_storm.SFileOpenFileEx.argtypes = [
    ctypes.c_void_p,
    ctypes.c_char_p,
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_void_p),
]
_storm.SFileOpenFileEx.restype = ctypes.c_bool

_storm.SFileGetFileSize.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32)]
_storm.SFileGetFileSize.restype = ctypes.c_uint32

_storm.SFileReadFile.argtypes = [
    ctypes.c_void_p,
    ctypes.c_void_p,
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_uint32),
    ctypes.c_void_p,
]
_storm.SFileReadFile.restype = ctypes.c_bool

_storm.SFileCloseFile.argtypes = [ctypes.c_void_p]
_storm.SFileCloseFile.restype = ctypes.c_bool

STREAM_FLAG_READ_ONLY = 0x00000100


# ---------------------------------------------------------------------------
# MPQ helpers
# ---------------------------------------------------------------------------

def _read_mpq_file(h_mpq: ctypes.c_void_p, mpq_path: str) -> bytes:
    """Read the full contents of a file from the MPQ archive."""
    h_file = ctypes.c_void_p()
    if not _storm.SFileOpenFileEx(h_mpq, mpq_path.encode("latin-1"), 0, ctypes.byref(h_file)):
        raise RuntimeError(f"Failed to open '{mpq_path}' inside MPQ")
    try:
        size = _storm.SFileGetFileSize(h_file, None)
        if size == 0xFFFFFFFF:
            raise RuntimeError(f"Failed to get size of '{mpq_path}'")
        buf = ctypes.create_string_buffer(size)
        read = ctypes.c_uint32(0)
        if not _storm.SFileReadFile(h_file, buf, size, ctypes.byref(read), None):
            raise RuntimeError(f"Failed to read '{mpq_path}'")
        return buf.raw[: read.value]
    finally:
        _storm.SFileCloseFile(h_file)


# ---------------------------------------------------------------------------
# MDX parser
# ---------------------------------------------------------------------------

class Texture:
    def __init__(self, replaceable_id: int, file_name: str, flags: int):
        self.replaceable_id = replaceable_id
        self.file_name = file_name
        self.flags = flags


class Layer:
    def __init__(self, filter_mode: int, shading_flags: int, texture_id: int,
                 texture_animation_id: int, coord_id: int, alpha: float):
        self.filter_mode = filter_mode
        self.shading_flags = shading_flags
        self.texture_id = texture_id
        self.texture_animation_id = texture_animation_id
        self.coord_id = coord_id
        self.alpha = alpha


class Material:
    def __init__(self, priority_plane: int, flags: int, layers: list):
        self.priority_plane = priority_plane
        self.flags = flags
        self.layers = layers


class Geoset:
    def __init__(self):
        self.vertex_positions = []   # list of (x, y, z)
        self.vertex_normals = []     # list of (nx, ny, nz)
        self.faces = []              # list of (i0, i1, i2)
        self.uvs = []                # list of (u, v)
        self.material_id = 0
        self.selection_group = 0
        self.selection_flags = 0


def _read_float3(data: bytes, offset: int):
    return struct.unpack("<3f", data[offset:offset + 12])


def _read_float2(data: bytes, offset: int):
    return struct.unpack("<2f", data[offset:offset + 8])


def _wc3_to_gltf_pos(pos):
    """Convert WC3 coordinates to glTF coordinates."""
    x, y, z = pos
    return (x, z, -y)


def _wc3_to_gltf_normal(nrm):
    """Convert WC3 normal to glTF normal (renormalize after axis swap)."""
    x, y, z = nrm
    nx, ny, nz = x, z, -y
    length = math.sqrt(nx * nx + ny * ny + nz * nz)
    if length > 0:
        return (nx / length, ny / length, nz / length)
    return (0.0, 1.0, 0.0)


def parse_mdx(data: bytes) -> dict:
    """Parse an MDX v800 file and return a dict with textures, materials, geosets."""
    if data[:4] != b"MDLX":
        raise ValueError("Not a valid MDX file (missing MDLX magic)")

    version = None
    textures = []
    materials = []
    geosets = []

    offset = 4
    while offset < len(data):
        if offset + 8 > len(data):
            break
        chunk_id = data[offset:offset + 4].decode("ascii", errors="replace")
        chunk_size = int.from_bytes(data[offset + 4:offset + 8], "little")
        chunk_data = data[offset + 8:offset + 8 + chunk_size]
        next_offset = offset + 8 + chunk_size

        if chunk_id == "VERS":
            version = int.from_bytes(chunk_data[:4], "little")
            if version != 800:
                # Still try to parse, but warn
                pass

        elif chunk_id == "TEXS":
            tex_offset = 0
            while tex_offset + 268 <= len(chunk_data):
                replaceable_id = int.from_bytes(chunk_data[tex_offset:tex_offset + 4], "little")
                file_name = chunk_data[tex_offset + 4:tex_offset + 264].split(b"\x00")[0].decode("latin-1", errors="replace")
                flags = int.from_bytes(chunk_data[tex_offset + 264:tex_offset + 268], "little")
                textures.append(Texture(replaceable_id, file_name, flags))
                tex_offset += 268

        elif chunk_id == "MTLS":
            mtl_offset = 0
            mtl_end = len(chunk_data)
            while mtl_offset + 4 <= mtl_end:
                inclusive_size = int.from_bytes(chunk_data[mtl_offset:mtl_offset + 4], "little")
                if mtl_offset + inclusive_size > mtl_end:
                    break
                mtl_data = chunk_data[mtl_offset:mtl_offset + inclusive_size]
                sub_off = 4
                priority_plane = int.from_bytes(mtl_data[sub_off:sub_off + 4], "little")
                flags = int.from_bytes(mtl_data[sub_off + 4:sub_off + 8], "little")
                sub_off += 8
                layers = []
                if mtl_data[sub_off:sub_off + 4] == b"LAYS":
                    layers_count = int.from_bytes(mtl_data[sub_off + 4:sub_off + 8], "little")
                    sub_off += 8
                    for _ in range(layers_count):
                        layer_inclusive = int.from_bytes(mtl_data[sub_off:sub_off + 4], "little")
                        filter_mode = int.from_bytes(mtl_data[sub_off + 4:sub_off + 8], "little")
                        shading_flags = int.from_bytes(mtl_data[sub_off + 8:sub_off + 12], "little")
                        texture_id = int.from_bytes(mtl_data[sub_off + 12:sub_off + 16], "little")
                        tex_anim_id = int.from_bytes(mtl_data[sub_off + 16:sub_off + 20], "little")
                        coord_id = int.from_bytes(mtl_data[sub_off + 20:sub_off + 24], "little")
                        alpha = struct.unpack("<f", mtl_data[sub_off + 24:sub_off + 28])[0]
                        layers.append(Layer(filter_mode, shading_flags, texture_id,
                                            tex_anim_id, coord_id, alpha))
                        sub_off += layer_inclusive
                materials.append(Material(priority_plane, flags, layers))
                mtl_offset += inclusive_size

        elif chunk_id == "GEOS":
            geos_offset = 0
            geos_end = len(chunk_data)
            while geos_offset + 4 <= geos_end:
                inclusive_size = int.from_bytes(chunk_data[geos_offset:geos_offset + 4], "little")
                if geos_offset + inclusive_size > geos_end:
                    break
                geoset_data = chunk_data[geos_offset:geos_offset + inclusive_size]
                g = Geoset()
                sub_off = 4
                g_end = len(geoset_data)
                while sub_off < g_end:
                    tag_bytes = geoset_data[sub_off:sub_off + 4]
                    if len(tag_bytes) < 4:
                        break
                    is_tag = all(32 <= b < 127 for b in tag_bytes)
                    if is_tag:
                        sub_id = tag_bytes.decode("ascii", errors="replace")
                        if sub_id == "VRTX":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            for i in range(count):
                                g.vertex_positions.append(_read_float3(geoset_data, sub_off + 8 + i * 12))
                            sub_off += 8 + count * 12
                        elif sub_id == "NRMS":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            for i in range(count):
                                g.vertex_normals.append(_read_float3(geoset_data, sub_off + 8 + i * 12))
                            sub_off += 8 + count * 12
                        elif sub_id == "PTYP":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            # We only care about face types; skip
                            sub_off += 8 + count * 4
                        elif sub_id == "PCNT":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            sub_off += 8 + count * 4
                        elif sub_id == "PVTX":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            for i in range(0, count, 3):
                                i0 = int.from_bytes(geoset_data[sub_off + 8 + i * 2:sub_off + 10 + i * 2], "little")
                                i1 = int.from_bytes(geoset_data[sub_off + 10 + i * 2:sub_off + 12 + i * 2], "little")
                                i2 = int.from_bytes(geoset_data[sub_off + 12 + i * 2:sub_off + 14 + i * 2], "little")
                                g.faces.append((i0, i1, i2))
                            sub_off += 8 + count * 2
                        elif sub_id == "GNDX":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            sub_off += 8 + count * 1
                        elif sub_id == "MTGC":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            sub_off += 8 + count * 4
                        elif sub_id == "MATS":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            sub_off += 8 + count * 4
                        elif sub_id == "UVAS":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            sub_off += 8
                            for _ in range(count):
                                uv_tag = geoset_data[sub_off:sub_off + 4].decode("ascii", errors="replace")
                                uv_count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                                if uv_tag == "UVBS":
                                    for i in range(uv_count):
                                        g.uvs.append(_read_float2(geoset_data, sub_off + 8 + i * 8))
                                    sub_off += 8 + uv_count * 8
                                else:
                                    sub_off += 8 + uv_count * 8
                        elif sub_id in ("TANG", "SKIN"):
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            skip = count * 4 if sub_id == "TANG" else count * 1
                            sub_off += 8 + skip
                        else:
                            # Unknown sub-tag: try to skip by reading size
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            sub_off += 8 + count * 4
                    else:
                        # Trailing fixed fields
                        g.material_id = int.from_bytes(geoset_data[sub_off:sub_off + 4], "little")
                        g.selection_group = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                        g.selection_flags = int.from_bytes(geoset_data[sub_off + 8:sub_off + 12], "little")
                        sub_off += 40  # skip bounds_radius + min_extent + max_extent
                        extents_count = int.from_bytes(geoset_data[sub_off:sub_off + 4], "little")
                        sub_off += 4 + extents_count * 28
                        # After extents, continue loop to catch UVAS etc.
                geosets.append(g)
                geos_offset += inclusive_size

        offset = next_offset

    return {
        "version": version,
        "textures": textures,
        "materials": materials,
        "geosets": geosets,
    }


# ---------------------------------------------------------------------------
# glTF 2.0 writer
# ---------------------------------------------------------------------------

def _pad4(n: int) -> int:
    return (n + 3) & ~3


def _blp_to_png(blp_data: bytes) -> bytes:
    """Convert BLP bytes to PNG bytes using Pillow's BLP plugin."""
    img = Image.open(BytesIO(blp_data))
    out = BytesIO()
    img.save(out, format="PNG")
    return out.getvalue()


def write_glb(mdx_data: dict, out_path: Path, h_mpq=None) -> None:
    """Write a static-mesh glTF 2.0 binary file from parsed MDX data."""
    geosets = mdx_data["geosets"]
    materials = mdx_data["materials"]
    textures = mdx_data["textures"]

    # Collect all vertex data
    all_positions = []
    all_normals = []
    all_uvs = []
    all_indices = []
    primitive_materials = []
    index_offset = 0

    for g in geosets:
        if not g.faces:
            continue
        vert_count = len(g.vertex_positions)
        pos_list = [_wc3_to_gltf_pos(p) for p in g.vertex_positions]
        nrm_list = [_wc3_to_gltf_normal(n) for n in g.vertex_normals]
        # UVs: if missing, fill with zeros; flip V for glTF (bottom-left origin)
        uv_list = []
        if len(g.uvs) == vert_count:
            for u, v in g.uvs:
                uv_list.append((u, 1.0 - v))
        else:
            uv_list = [(0.0, 0.0)] * vert_count

        all_positions.extend(pos_list)
        all_normals.extend(nrm_list)
        all_uvs.extend(uv_list)

        for i0, i1, i2 in g.faces:
            all_indices.append((index_offset + i0, index_offset + i1, index_offset + i2))
        index_offset += vert_count

        primitive_materials.append(g.material_id)

    if not all_positions:
        raise ValueError("No geometry found in MDX")

    # Build binary buffer
    buf = BytesIO()

    # Positions accessor (offset 0)
    pos_offset = buf.tell()
    for x, y, z in all_positions:
        buf.write(struct.pack("<3f", x, y, z))
    pos_length = buf.tell() - pos_offset

    # Normals accessor
    nrm_offset = buf.tell()
    for x, y, z in all_normals:
        buf.write(struct.pack("<3f", x, y, z))
    nrm_length = buf.tell() - nrm_offset

    # UVs accessor
    uv_offset = buf.tell()
    for u, v in all_uvs:
        buf.write(struct.pack("<2f", u, v))
    uv_length = buf.tell() - uv_offset

    # Indices accessor
    idx_offset = buf.tell()
    for i0, i1, i2 in all_indices:
        buf.write(struct.pack("<HHH", i0, i1, i2))
    idx_length = buf.tell() - idx_offset

    buffer_data = buf.getvalue()
    buffer_length = len(buffer_data)

    # Build accessors / bufferViews / primitives
    accessors = [
        {
            "bufferView": 0,
            "componentType": 5126,  # FLOAT
            "count": len(all_positions),
            "type": "VEC3",
            "max": [max(p[i] for p in all_positions) for i in range(3)],
            "min": [min(p[i] for p in all_positions) for i in range(3)],
        },
        {
            "bufferView": 1,
            "componentType": 5126,  # FLOAT
            "count": len(all_normals),
            "type": "VEC3",
        },
        {
            "bufferView": 2,
            "componentType": 5126,  # FLOAT
            "count": len(all_uvs),
            "type": "VEC2",
        },
        {
            "bufferView": 3,
            "componentType": 5123,  # UNSIGNED_SHORT
            "count": len(all_indices) * 3,
            "type": "SCALAR",
        },
    ]

    buffer_views = [
        {"buffer": 0, "byteOffset": pos_offset, "byteLength": pos_length, "target": 34962},
        {"buffer": 0, "byteOffset": nrm_offset, "byteLength": nrm_length, "target": 34962},
        {"buffer": 0, "byteOffset": uv_offset, "byteLength": uv_length, "target": 34962},
        {"buffer": 0, "byteOffset": idx_offset, "byteLength": idx_length, "target": 34963},
    ]

    # Build materials (only use first layer of each material)
    gltf_materials = []
    gltf_textures = []
    gltf_images = []
    texture_index_map = {}  # MDX texture_id -> glTF texture index
    textures_out_dir = out_path.parent / "textures"

    for mat_idx, mat in enumerate(materials):
        pbr = {
            "baseColorFactor": [1.0, 1.0, 1.0, 1.0],
            "metallicFactor": 0.0,
            "roughnessFactor": 0.9,
        }
        material_def = {"pbrMetallicRoughness": pbr, "doubleSided": True}

        if mat.layers:
            layer = mat.layers[0]
            tex_id = layer.texture_id
            if 0 <= tex_id < len(textures):
                tex = textures[tex_id]
                if tex.file_name and tex.replaceable_id == 0:
                    if tex_id not in texture_index_map:
                        texture_index_map[tex_id] = len(gltf_textures)
                        # Derive PNG filename from BLP path
                        base_name = os.path.basename(tex.file_name.replace("\\", "/")).replace(".blp", ".png")
                        image_uri = f"textures/{base_name}"
                        gltf_images.append({"uri": image_uri})
                        gltf_textures.append({"source": len(gltf_images) - 1, "sampler": 0})

                        # Extract BLP -> PNG if MPQ handle is available
                        if h_mpq is not None and _HAS_PILLOW:
                            mpq_tex_path = tex.file_name.replace("/", "\\")
                            try:
                                blp_data = _read_mpq_file(h_mpq, mpq_tex_path)
                                png_data = _blp_to_png(blp_data)
                                textures_out_dir.mkdir(parents=True, exist_ok=True)
                                png_path = textures_out_dir / base_name
                                png_path.write_bytes(png_data)
                                print(f"  Extracted texture: {mpq_tex_path} -> {png_path}")
                            except Exception as exc:
                                print(f"  WARNING: Failed to extract texture {mpq_tex_path}: {exc}", file=sys.stderr)

                    material_def["pbrMetallicRoughness"]["baseColorTexture"] = {
                        "index": texture_index_map[tex_id],
                        "texCoord": 0,
                    }

        gltf_materials.append(material_def)

    # Build mesh primitives
    primitives = []
    for prim_idx, mat_id in enumerate(primitive_materials):
        prim = {
            "attributes": {
                "POSITION": 0,
                "NORMAL": 1,
                "TEXCOORD_0": 2,
            },
            "indices": 3,
            "mode": 4,  # TRIANGLES
        }
        if 0 <= mat_id < len(gltf_materials):
            prim["material"] = mat_id
        primitives.append(prim)

    # Because we merged all geosets into one buffer, we have ONE mesh with multiple primitives
    # But each primitive uses the same index buffer, which is wrong!
    # We need separate primitive index ranges.
    # Actually, since we merged everything, we should make ONE mesh with ONE primitive.
    # But materials differ per geoset. Let's create one primitive per geoset with proper offsets.

    # Recalculate with per-primitive index ranges
    primitives = []
    idx_cursor = idx_offset
    for g in geosets:
        if not g.faces:
            continue
        face_count = len(g.faces)
        prim_idx_offset = idx_cursor
        prim_idx_length = face_count * 3 * 2
        idx_cursor += prim_idx_length

        prim = {
            "attributes": {
                "POSITION": 0,
                "NORMAL": 1,
                "TEXCOORD_0": 2,
            },
            "indices": len(accessors),
            "mode": 4,
        }
        if 0 <= g.material_id < len(gltf_materials):
            prim["material"] = g.material_id

        accessors.append({
            "bufferView": len(buffer_views),
            "componentType": 5123,
            "count": face_count * 3,
            "type": "SCALAR",
        })
        buffer_views.append({
            "buffer": 0,
            "byteOffset": prim_idx_offset,
            "byteLength": prim_idx_length,
            "target": 34963,
        })
        primitives.append(prim)

    gltf = {
        "asset": {"version": "2.0", "generator": "wc3-mdx-to-gltf"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": primitives}],
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{"byteLength": buffer_length}],
    }

    if gltf_materials:
        gltf["materials"] = gltf_materials
    if gltf_textures:
        gltf["textures"] = gltf_textures
    if gltf_images:
        gltf["images"] = gltf_images
    if gltf_textures:
        gltf["samplers"] = [{"magFilter": 9729, "minFilter": 9987}]

    json_bytes = json.dumps(gltf, separators=(",", ":")).encode("utf-8")
    json_length = _pad4(len(json_bytes))
    bin_length = _pad4(buffer_length)

    total_length = 12 + 8 + json_length + 8 + bin_length

    glb = BytesIO()
    glb.write(struct.pack("<4sII", b"glTF", 2, total_length))
    glb.write(struct.pack("<I", json_length))
    glb.write(struct.pack("<I", 0x4E4F534A))  # JSON
    glb.write(json_bytes)
    glb.write(b"\x20" * (json_length - len(json_bytes)))
    glb.write(struct.pack("<I", bin_length))
    glb.write(struct.pack("<I", 0x004E4942))  # BIN
    glb.write(buffer_data)
    glb.write(b"\x00" * (bin_length - buffer_length))

    out_path.write_bytes(glb.getvalue())
    print(f"Wrote {out_path} ({total_length} bytes)")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(description="Convert WC3 MDX to glTF 2.0 (.glb)")
    parser.add_argument("--mpq", required=True, help="Path to the MPQ archive.")
    parser.add_argument("--mdx", required=True, help="Internal MPQ path to the MDX file (e.g. Units/Human/Peasant/peasant.mdx)")
    parser.add_argument("--out", required=True, help="Output .glb file path.")
    args = parser.parse_args()

    mpq_path = Path(args.mpq)
    if not mpq_path.exists():
        print(f"ERROR: MPQ file not found: {mpq_path}", file=sys.stderr)
        return 1

    h_mpq = ctypes.c_void_p()
    if not _storm.SFileOpenArchive(str(mpq_path).encode("utf-8"), 0, STREAM_FLAG_READ_ONLY, ctypes.byref(h_mpq)):
        print(f"ERROR: Failed to open MPQ archive: {mpq_path}", file=sys.stderr)
        return 1

    try:
        mdx_bytes = _read_mpq_file(h_mpq, args.mdx.replace("/", "\\"))
        print(f"Read {len(mdx_bytes)} bytes from {args.mdx}")
        mdx_parsed = parse_mdx(mdx_bytes)
        print(f"Parsed MDX v{mdx_parsed['version']} with {len(mdx_parsed['geosets'])} geoset(s), "
              f"{len(mdx_parsed['materials'])} material(s), {len(mdx_parsed['textures'])} texture(s)")
        out_path = Path(args.out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        write_glb(mdx_parsed, out_path)
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return 1
    finally:
        _storm.SFileCloseArchive(h_mpq)

    return 0


if __name__ == "__main__":
    sys.exit(main())
