#!/usr/bin/env python3
"""
Minimal WC3 MDX → glTF 2.0 converter.
Parses MDX version 800 static mesh + UVs + first texture layer + bones/skinning + animations.
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


class Sequence:
    def __init__(self, name: str, start_ms: int, end_ms: int, move_speed: float, loop: bool):
        self.name = name
        self.start_ms = start_ms
        self.end_ms = end_ms
        self.move_speed = move_speed
        self.loop = loop


class Bone:
    def __init__(self, name: str, object_id: int, parent_id: int, flags: int):
        self.name = name
        self.object_id = object_id
        self.parent_id = parent_id
        self.flags = flags
        self.translations = []   # list of (time_ms, (x, y, z), interp_type)
        self.rotations = []      # list of (time_ms, (x, y, z, w), interp_type)
        self.scales = []         # list of (time_ms, (x, y, z), interp_type)


class Geoset:
    def __init__(self):
        self.vertex_positions = []   # list of (x, y, z)
        self.vertex_normals = []     # list of (nx, ny, nz)
        self.faces = []              # list of (i0, i1, i2)
        self.uvs = []                # list of (u, v)
        self.material_id = 0
        self.selection_group = 0
        self.selection_flags = 0
        self.vertex_groups = []      # GNDX: uint8 matrix group per vertex
        self.matrix_group_counts = []  # MTGC: uint32 count per matrix group
        self.matrix_indices = []     # MATS: uint32 bone indices (flat)


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


def _wc3_quat_to_gltf(quat):
    """Convert WC3 quaternion to glTF quaternion (axis swap)."""
    x, y, z, w = quat
    return (x, z, -y, w)


def _parse_track_data(node_data: bytes, track_offset: int, track_tag: str):
    """Parse MDX keyframe track starting at track_offset in node_data.
    Format: tag(4) + count(4) + type(4) + global_seq_id(4) + keyframes...
    Returns (keyframes, track_byte_length).
    """
    if track_offset + 16 > len(node_data):
        return [], 0

    count = int.from_bytes(node_data[track_offset + 4:track_offset + 8], "little")
    interp_type = int.from_bytes(node_data[track_offset + 8:track_offset + 12], "little")
    # global_seq_id = int.from_bytes(node_data[track_offset + 12:track_offset + 16], "little", signed=True)

    if track_tag in ("KGTR", "KGSC"):  # translation or scale
        linear_size = 16   # time(4) + value(3*4)
        hermite_size = 40  # time + value + in-tangent(3f) + out-tangent(3f)
    elif track_tag == "KGRT":  # rotation
        linear_size = 20   # time(4) + value(4*4)
        hermite_size = 52  # time + value + in-tangent(4f) + out-tangent(4f)
    else:
        return [], 0

    if interp_type in (0, 1):  # NONE, LINEAR
        keyframe_size = linear_size
    elif interp_type in (2, 3):  # HERMITE, BEZIER
        keyframe_size = hermite_size
    else:
        return [], 0

    data_offset = track_offset + 16
    expected_end = data_offset + count * keyframe_size
    if expected_end > len(node_data):
        return [], 0

    keyframes = []
    for _ in range(count):
        time_ms = int.from_bytes(node_data[data_offset:data_offset + 4], "little")
        if track_tag == "KGTR":
            x, y, z = struct.unpack("<3f", node_data[data_offset + 4:data_offset + 16])
            value = _wc3_to_gltf_pos((x, y, z))
        elif track_tag == "KGRT":
            x, y, z, w = struct.unpack("<4f", node_data[data_offset + 4:data_offset + 20])
            value = _wc3_quat_to_gltf((x, y, z, w))
            # Normalize quaternion
            mag = math.sqrt(sum(v * v for v in value))
            if mag > 0:
                value = tuple(v / mag for v in value)
        elif track_tag == "KGSC":
            x, y, z = struct.unpack("<3f", node_data[data_offset + 4:data_offset + 16])
            value = (x, y, z)
        else:
            value = None

        if value is not None:
            keyframes.append((time_ms, value, interp_type))
        data_offset += keyframe_size

    track_byte_length = 16 + count * keyframe_size
    return keyframes, track_byte_length


def _parse_bone_nodes(chunk_data: bytes) -> list:
    """Parse BONE chunk entries including animation tracks."""
    bones = []
    i = 0
    while i + 96 <= len(chunk_data):
        size = int.from_bytes(chunk_data[i:i + 4], "little")
        if size < 96 or i + size > len(chunk_data):
            i += 4
            continue
        name = chunk_data[i + 4:i + 84].split(b"\x00")[0].decode("ascii", errors="replace")
        obj_id = int.from_bytes(chunk_data[i + 84:i + 88], "little")
        parent = int.from_bytes(chunk_data[i + 88:i + 92], "little")
        flags = int.from_bytes(chunk_data[i + 92:i + 96], "little")
        # Sanity checks
        if obj_id < 500 and (parent == 0xFFFFFFFF or parent < 500) and flags in (0, 256, 512, 1024, 2048):
            bone = Bone(name, obj_id, parent, flags)
            # Parse track chunks after the 96-byte header
            track_offset = 96
            while track_offset + 16 <= size:
                tag = chunk_data[i + track_offset:i + track_offset + 4].decode("ascii", errors="replace")
                if tag not in ("KGTR", "KGRT", "KGSC"):
                    break
                kfs, track_byte_length = _parse_track_data(chunk_data, i + track_offset, tag)
                if tag == "KGTR":
                    bone.translations.extend(kfs)
                elif tag == "KGRT":
                    bone.rotations.extend(kfs)
                elif tag == "KGSC":
                    bone.scales.extend(kfs)
                track_offset += track_byte_length

            bones.append(bone)
            i += size
            continue
        i += 4
    return bones


def parse_mdx(data: bytes) -> dict:
    """Parse an MDX v800 file and return a dict with textures, materials, geosets, bones, pivots, sequences."""
    if data[:4] != b"MDLX":
        raise ValueError("Not a valid MDX file (missing MDLX magic)")

    version = None
    textures = []
    materials = []
    geosets = []
    bones = []
    pivots = []
    sequences = []

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
                            g.vertex_groups = list(geoset_data[sub_off + 8:sub_off + 8 + count])
                            sub_off += 8 + count * 1
                        elif sub_id == "MTGC":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            for i in range(count):
                                g.matrix_group_counts.append(
                                    int.from_bytes(geoset_data[sub_off + 8 + i * 4:sub_off + 12 + i * 4], "little")
                                )
                            sub_off += 8 + count * 4
                        elif sub_id == "MATS":
                            count = int.from_bytes(geoset_data[sub_off + 4:sub_off + 8], "little")
                            for i in range(count):
                                g.matrix_indices.append(
                                    int.from_bytes(geoset_data[sub_off + 8 + i * 4:sub_off + 12 + i * 4], "little")
                                )
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

        elif chunk_id == "BONE":
            bones = _parse_bone_nodes(chunk_data)

        elif chunk_id == "PIVT":
            count = len(chunk_data) // 12
            for i in range(count):
                pivots.append(_read_float3(chunk_data, i * 12))

        elif chunk_id == "SEQS":
            seq_offset = 0
            while seq_offset + 132 <= len(chunk_data):
                name = chunk_data[seq_offset:seq_offset + 80].split(b"\x00")[0].decode("ascii", errors="replace")
                start_ms = int.from_bytes(chunk_data[seq_offset + 80:seq_offset + 84], "little")
                end_ms = int.from_bytes(chunk_data[seq_offset + 84:seq_offset + 88], "little")
                move_speed = struct.unpack("<f", chunk_data[seq_offset + 88:seq_offset + 92])[0]
                no_loop = int.from_bytes(chunk_data[seq_offset + 92:seq_offset + 96], "little")
                sequences.append(Sequence(name, start_ms, end_ms, move_speed, loop=(no_loop == 0)))
                seq_offset += 132

        offset = next_offset

    return {
        "version": version,
        "textures": textures,
        "materials": materials,
        "geosets": geosets,
        "bones": bones,
        "pivots": pivots,
        "sequences": sequences,
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
    bones = mdx_data.get("bones", [])
    pivots = mdx_data.get("pivots", [])
    sequences = mdx_data.get("sequences", [])

    # Build object_id -> joint_index map for bones
    object_id_to_joint = {}
    for joint_idx, bone in enumerate(bones):
        object_id_to_joint[bone.object_id] = joint_idx

    # Build bone parent relationships (only among bones)
    bone_obj_ids = {b.object_id for b in bones}
    joint_children = [[] for _ in bones]
    root_joint_indices = []
    for joint_idx, bone in enumerate(bones):
        if bone.parent_id in bone_obj_ids:
            parent_joint = object_id_to_joint[bone.parent_id]
            joint_children[parent_joint].append(joint_idx)
        else:
            root_joint_indices.append(joint_idx)

    # Collect all vertex data + skinning data
    all_positions = []
    all_normals = []
    all_uvs = []
    all_joints = []   # list of (j0, j1, j2, j3)
    all_weights = []  # list of (w0, w1, w2, w3)
    all_indices = []
    primitive_materials = []
    index_offset = 0

    for g in geosets:
        if not g.faces:
            continue
        vert_count = len(g.vertex_positions)
        pos_list = [_wc3_to_gltf_pos(p) for p in g.vertex_positions]
        nrm_list = [_wc3_to_gltf_normal(n) for n in g.vertex_normals]
        uv_list = []
        if len(g.uvs) == vert_count:
            for u, v in g.uvs:
                uv_list.append((u, 1.0 - v))
        else:
            uv_list = [(0.0, 0.0)] * vert_count

        all_positions.extend(pos_list)
        all_normals.extend(nrm_list)
        all_uvs.extend(uv_list)

        # Skinning data per vertex
        for v_idx in range(vert_count):
            if v_idx < len(g.vertex_groups) and g.matrix_group_counts and g.matrix_indices:
                group_idx = g.vertex_groups[v_idx]
                group_start = sum(g.matrix_group_counts[:group_idx])
                group_count = g.matrix_group_counts[group_idx]
                bone_obj_ids_for_v = g.matrix_indices[group_start:group_start + group_count]
                joint_indices = []
                for obj_id in bone_obj_ids_for_v:
                    if obj_id in object_id_to_joint:
                        joint_indices.append(object_id_to_joint[obj_id])
                    else:
                        # Fallback: map to root joint if available, else 0
                        joint_indices.append(root_joint_indices[0] if root_joint_indices else 0)
                # Pad to 4, distribute weights evenly
                while len(joint_indices) < 4:
                    joint_indices.append(0)
                weight = 1.0 / len(bone_obj_ids_for_v) if bone_obj_ids_for_v else 1.0
                weights = [weight if i < len(bone_obj_ids_for_v) else 0.0 for i in range(4)]
                # Renormalize to be safe
                total_w = sum(weights)
                if total_w > 0:
                    weights = [w / total_w for w in weights]
                all_joints.append(tuple(joint_indices[:4]))
                all_weights.append(tuple(weights))
            else:
                # No skinning data: bind to root joint (or joint 0)
                all_joints.append((0, 0, 0, 0))
                all_weights.append((1.0, 0.0, 0.0, 0.0))

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

    # JOINTS_0 accessor (UNSIGNED_BYTE x 4)
    jnt_offset = buf.tell()
    for j0, j1, j2, j3 in all_joints:
        buf.write(struct.pack("<4B", j0, j1, j2, j3))
    jnt_length = buf.tell() - jnt_offset

    # WEIGHTS_0 accessor (FLOAT x 4)
    wgt_offset = buf.tell()
    for w0, w1, w2, w3 in all_weights:
        buf.write(struct.pack("<4f", w0, w1, w2, w3))
    wgt_length = buf.tell() - wgt_offset

    # Indices accessor
    idx_offset = buf.tell()
    for i0, i1, i2 in all_indices:
        buf.write(struct.pack("<HHH", i0, i1, i2))
    idx_length = buf.tell() - idx_offset

    buffer_data = buf.getvalue()
    buffer_length = len(buffer_data)

    # Build accessors / bufferViews
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
            "componentType": 5121,  # UNSIGNED_BYTE
            "count": len(all_joints),
            "type": "VEC4",
        },
        {
            "bufferView": 4,
            "componentType": 5126,  # FLOAT
            "count": len(all_weights),
            "type": "VEC4",
        },
    ]

    buffer_views = [
        {"buffer": 0, "byteOffset": pos_offset, "byteLength": pos_length, "target": 34962},
        {"buffer": 0, "byteOffset": nrm_offset, "byteLength": nrm_length, "target": 34962},
        {"buffer": 0, "byteOffset": uv_offset, "byteLength": uv_length, "target": 34962},
        {"buffer": 0, "byteOffset": jnt_offset, "byteLength": jnt_length, "target": 34962},
        {"buffer": 0, "byteOffset": wgt_offset, "byteLength": wgt_length, "target": 34962},
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

    # Build per-primitive index ranges
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
                "JOINTS_0": 3,
                "WEIGHTS_0": 4,
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

    # Build bone nodes
    bone_nodes = []
    for joint_idx, bone in enumerate(bones):
        pivot_gl = _wc3_to_gltf_pos(pivots[bone.object_id]) if bone.object_id < len(pivots) else (0.0, 0.0, 0.0)
        if bone.parent_id in bone_obj_ids and bone.parent_id < len(pivots):
            parent_pivot_gl = _wc3_to_gltf_pos(pivots[bone.parent_id])
            local_translation = (
                pivot_gl[0] - parent_pivot_gl[0],
                pivot_gl[1] - parent_pivot_gl[1],
                pivot_gl[2] - parent_pivot_gl[2],
            )
        else:
            local_translation = pivot_gl

        node = {
            "name": bone.name,
            "translation": [local_translation[0], local_translation[1], local_translation[2]],
        }
        if joint_children[joint_idx]:
            node["children"] = joint_children[joint_idx]
        bone_nodes.append(node)

    # Build inverse bind matrices
    ibm_data = []
    for bone in bones:
        pivot_gl = _wc3_to_gltf_pos(pivots[bone.object_id]) if bone.object_id < len(pivots) else (0.0, 0.0, 0.0)
        # Inverse of translation-only matrix: translate by -pivot
        ibm_data.extend([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -pivot_gl[0], -pivot_gl[1], -pivot_gl[2], 1.0,
        ])

    # Append IBM to the binary buffer
    buf2 = BytesIO()
    buf2.write(buffer_data)
    ibm_offset = buf2.tell()
    for val in ibm_data:
        buf2.write(struct.pack("<f", val))
    ibm_length = buf2.tell() - ibm_offset

    # Add IBM accessor and bufferView
    ibm_accessor_idx = len(accessors)
    accessors.append({
        "bufferView": len(buffer_views),
        "componentType": 5126,  # FLOAT
        "count": len(bones),
        "type": "MAT4",
    })
    buffer_views.append({
        "buffer": 0,
        "byteOffset": ibm_offset,
        "byteLength": ibm_length,
    })

    # Build animations
    animations = []
    if sequences and bones:
        for seq in sequences:
            samplers = []
            channels = []
            for bone_idx, bone in enumerate(bones):
                for track_list, path_name, vec_size in [
                    (bone.translations, "translation", 3),
                    (bone.rotations, "rotation", 4),
                    (bone.scales, "scale", 3),
                ]:
                    # Filter keyframes inside sequence range
                    kfs = [(t, v, interp) for t, v, interp in track_list if seq.start_ms <= t <= seq.end_ms]
                    if not kfs:
                        continue

                    kfs.sort(key=lambda x: x[0])

                    # Rebase times to seconds
                    times = [(t - seq.start_ms) / 1000.0 for t, v, interp in kfs]
                    values = [v for t, v, interp in kfs]

                    # Write input accessor
                    input_offset = buf2.tell()
                    for t in times:
                        buf2.write(struct.pack("<f", t))
                    input_length = buf2.tell() - input_offset

                    input_acc_idx = len(accessors)
                    accessors.append({
                        "bufferView": len(buffer_views),
                        "componentType": 5126,  # FLOAT
                        "count": len(times),
                        "type": "SCALAR",
                        "min": [min(times)],
                        "max": [max(times)],
                    })
                    buffer_views.append({
                        "buffer": 0,
                        "byteOffset": input_offset,
                        "byteLength": input_length,
                    })

                    # Write output accessor
                    output_offset = buf2.tell()
                    if vec_size == 4:
                        for x, y, z, w in values:
                            buf2.write(struct.pack("<4f", x, y, z, w))
                        out_type = "VEC4"
                    else:
                        for x, y, z in values:
                            buf2.write(struct.pack("<3f", x, y, z))
                        out_type = "VEC3"
                    output_length = buf2.tell() - output_offset

                    output_acc_idx = len(accessors)
                    accessors.append({
                        "bufferView": len(buffer_views),
                        "componentType": 5126,  # FLOAT
                        "count": len(values),
                        "type": out_type,
                    })
                    buffer_views.append({
                        "buffer": 0,
                        "byteOffset": output_offset,
                        "byteLength": output_length,
                    })

                    # Determine interpolation type for glTF
                    interp_types = {interp for t, v, interp in kfs}
                    if all(i == 0 for i in interp_types):
                        gltf_interp = "STEP"
                    else:
                        gltf_interp = "LINEAR"

                    sampler_idx = len(samplers)
                    samplers.append({
                        "input": input_acc_idx,
                        "interpolation": gltf_interp,
                        "output": output_acc_idx,
                    })
                    channels.append({
                        "sampler": sampler_idx,
                        "target": {
                            "node": 1 + bone_idx,
                            "path": path_name,
                        },
                    })

            if channels:
                animations.append({
                    "name": seq.name,
                    "channels": channels,
                    "samplers": samplers,
                })

    buffer_data = buf2.getvalue()
    buffer_length = len(buffer_data)

    # Build glTF document
    mesh_node = {"mesh": 0}
    if bones:
        mesh_node["skin"] = 0

    scene_nodes = [0]  # mesh node at index 0
    if bones:
        # Root bone nodes come after mesh node
        root_bone_node_indices = [1 + i for i in root_joint_indices]
        scene_nodes.extend(root_bone_node_indices)

    nodes = [mesh_node] + bone_nodes

    gltf = {
        "asset": {"version": "2.0", "generator": "wc3-mdx-to-gltf"},
        "scene": 0,
        "scenes": [{"nodes": scene_nodes}],
        "nodes": nodes,
        "meshes": [{"primitives": primitives}],
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{"byteLength": buffer_length}],
    }

    if bones:
        gltf["skins"] = [{
            "inverseBindMatrices": ibm_accessor_idx,
            "joints": list(range(1, 1 + len(bones))),  # node indices of bone nodes
        }]

    if animations:
        gltf["animations"] = animations

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
    if sequences:
        print(f"  Sequences: {len(sequences)}, Animations: {len(animations)}")
        for anim in animations:
            print(f"    Animation '{anim['name']}': {len(anim['channels'])} channels, {len(anim['samplers'])} samplers")

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
              f"{len(mdx_parsed['materials'])} material(s), {len(mdx_parsed['textures'])} texture(s), "
              f"{len(mdx_parsed.get('bones', []))} bone(s), {len(mdx_parsed.get('pivots', []))} pivot(s), "
              f"{len(mdx_parsed.get('sequences', []))} sequence(s)")
        out_path = Path(args.out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        write_glb(mdx_parsed, out_path, h_mpq=h_mpq)
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
