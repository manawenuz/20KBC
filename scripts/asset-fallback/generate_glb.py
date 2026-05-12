#!/usr/bin/env python3
"""
Generate minimal valid .glb files for 20KBC fallback models.
Uses raw glTF 2.0 binary format — no external dependencies.
"""

import struct
import json
import os


def write_glb(json_data: dict, bin_data: bytes) -> bytes:
    """Pack JSON + binary into a valid .glb file."""
    json_str = json.dumps(json_data, separators=(',', ':')).encode('utf-8')
    # Pad JSON to 4-byte boundary
    json_pad = (4 - (len(json_str) % 4)) % 4
    json_str += b' ' * json_pad

    # Pad binary to 4-byte boundary
    bin_pad = (4 - (len(bin_data) % 4)) % 4
    bin_data += b'\x00' * bin_pad

    json_chunk_len = len(json_str)
    bin_chunk_len = len(bin_data)

    total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len

    header = struct.pack('<III', 0x46546C67, 2, total_len)
    json_chunk_header = struct.pack('<II', json_chunk_len, 0x4E4F534A)
    bin_chunk_header = struct.pack('<II', bin_chunk_len, 0x004E4942)

    return header + json_chunk_header + json_str + bin_chunk_header + bin_data


def build_mesh(parts, color):
    """
    parts: list of (vertices, indices) where vertices is list of (x,y,z,nx,ny,nz)
    Returns (all_vertices_flat, all_indices, index_offset)
    """
    all_vertices = []
    all_indices = []
    idx_offset = 0
    for vertices, indices in parts:
        all_vertices.extend(vertices)
        all_indices.extend([i + idx_offset for i in indices])
        idx_offset += len(vertices)
    return all_vertices, all_indices


def box(min_x, min_y, min_z, max_x, max_y, max_z):
    """Return (vertices, indices) for an axis-aligned box."""
    # 8 corners
    corners = [
        (min_x, min_y, min_z), (max_x, min_y, min_z),
        (max_x, max_y, min_z), (min_x, max_y, min_z),
        (min_x, min_y, max_z), (max_x, min_y, max_z),
        (max_x, max_y, max_z), (min_x, max_y, max_z),
    ]
    # face normals
    normals = [
        (0, 0, -1),  # front
        (0, 0, 1),   # back
        (-1, 0, 0),  # left
        (1, 0, 0),   # right
        (0, 1, 0),   # top
        (0, -1, 0),  # bottom
    ]
    # Each face: 4 vertices (triangulated as 0,1,2 and 0,2,3)
    face_indices = [
        [0, 1, 2, 3],  # front  z-
        [5, 4, 7, 6],  # back   z+
        [4, 0, 3, 7],  # left   x-
        [1, 5, 6, 2],  # right  x+
        [3, 2, 6, 7],  # top    y+
        [4, 5, 1, 0],  # bottom y-
    ]
    vertices = []
    indices = []
    idx = 0
    for fi, quad in enumerate(face_indices):
        nx, ny, nz = normals[fi]
        for qi in quad:
            x, y, z = corners[qi]
            vertices.append((x, y, z, nx, ny, nz))
        indices.extend([idx, idx+1, idx+2, idx, idx+2, idx+3])
        idx += 4
    return vertices, indices


def make_glb(parts, color, name):
    """parts: list of (vertices, indices); color: (r,g,b,a)"""
    vertices, indices = build_mesh(parts, color)

    # Vertex buffer: 3 floats pos + 3 floats normal = 6 floats = 24 bytes per vertex
    # Interleaved: pos (3), normal (3)
    vertex_data = b''.join(
        struct.pack('<6f', x, y, z, nx, ny, nz)
        for (x, y, z, nx, ny, nz) in vertices
    )

    # Index buffer: uint16
    index_data = b''.join(struct.pack('<H', i) for i in indices)

    bin_data = vertex_data + index_data
    vertex_len = len(vertex_data)
    index_len = len(index_data)

    json_data = {
        "asset": {"version": "2.0", "generator": "20kbc-fallback-gen"},
        "scene": 0,
        "scenes": [{"name": name, "nodes": [0]}],
        "nodes": [{"name": name, "mesh": 0}],
        "meshes": [{
            "name": name,
            "primitives": [{
                "attributes": {
                    "POSITION": 0,
                    "NORMAL": 1
                },
                "indices": 2,
                "mode": 4,  # TRIANGLES
                "material": 0
            }]
        }],
        "materials": [{
            "name": name + "_mat",
            "pbrMetallicRoughness": {
                "baseColorFactor": list(color),
                "metallicFactor": 0.0,
                "roughnessFactor": 0.8
            }
        }],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,  # FLOAT
                "count": len(vertices),
                "type": "VEC3",
                "max": [max(v[0] for v in vertices), max(v[1] for v in vertices), max(v[2] for v in vertices)],
                "min": [min(v[0] for v in vertices), min(v[1] for v in vertices), min(v[2] for v in vertices)],
            },
            {
                "bufferView": 0,
                "byteOffset": 12,
                "componentType": 5126,  # FLOAT
                "count": len(vertices),
                "type": "VEC3",
            },
            {
                "bufferView": 1,
                "componentType": 5123,  # UNSIGNED_SHORT
                "count": len(indices),
                "type": "SCALAR",
            },
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": vertex_len,
                "byteStride": 24,
            },
            {
                "buffer": 0,
                "byteOffset": vertex_len,
                "byteLength": index_len,
            },
        ],
        "buffers": [{"byteLength": len(bin_data)}]
    }

    return write_glb(json_data, bin_data)


def generate_peasant():
    """Simple humanoid: box head, box torso, box arms, box legs."""
    parts = []
    # Head
    parts.append(box(-0.15, 1.5, -0.15, 0.15, 1.8, 0.15))
    # Torso
    parts.append(box(-0.25, 0.8, -0.15, 0.25, 1.5, 0.15))
    # Left arm
    parts.append(box(-0.45, 0.8, -0.1, -0.28, 1.45, 0.1))
    # Right arm
    parts.append(box(0.28, 0.8, -0.1, 0.45, 1.45, 0.1))
    # Left leg
    parts.append(box(-0.22, 0.0, -0.1, -0.05, 0.8, 0.1))
    # Right leg
    parts.append(box(0.05, 0.0, -0.1, 0.22, 0.8, 0.1))
    return make_glb(parts, (0.76, 0.60, 0.42, 1.0), "Peasant")


def generate_wolf():
    """Simple quadruped: body, neck+head, 4 legs, tail."""
    parts = []
    # Body
    parts.append(box(-0.25, 0.5, -0.6, 0.25, 0.9, 0.5))
    # Neck
    parts.append(box(-0.15, 0.7, 0.4, 0.15, 1.1, 0.8))
    # Head
    parts.append(box(-0.18, 0.85, 0.75, 0.18, 1.15, 1.15))
    # Snout
    parts.append(box(-0.1, 0.9, 1.1, 0.1, 1.05, 1.35))
    # Front left leg
    parts.append(box(-0.22, 0.0, 0.35, -0.08, 0.55, 0.5))
    # Front right leg
    parts.append(box(0.08, 0.0, 0.35, 0.22, 0.55, 0.5))
    # Back left leg
    parts.append(box(-0.22, 0.0, -0.5, -0.08, 0.55, -0.35))
    # Back right leg
    parts.append(box(0.08, 0.0, -0.5, 0.22, 0.55, -0.35))
    # Tail
    parts.append(box(-0.06, 0.55, -0.9, 0.06, 0.7, -0.55))
    return make_glb(parts, (0.45, 0.40, 0.38, 1.0), "Wolf")


def generate_tree():
    """Simple tree: trunk + 3 canopy spheres approximated as boxes."""
    parts = []
    # Trunk
    parts.append(box(-0.12, 0.0, -0.12, 0.12, 1.2, 0.12))
    # Canopy center
    parts.append(box(-0.5, 1.0, -0.5, 0.5, 1.8, 0.5))
    # Canopy left
    parts.append(box(-0.7, 1.2, -0.25, -0.2, 1.7, 0.25))
    # Canopy right
    parts.append(box(0.2, 1.2, -0.25, 0.7, 1.7, 0.25))
    # Canopy top
    parts.append(box(-0.35, 1.7, -0.35, 0.35, 2.1, 0.35))
    return make_glb(parts, (0.13, 0.55, 0.13, 1.0), "Tree")


def generate_stone():
    """Simple rock cluster: 4 overlapping irregular boxes."""
    parts = []
    # Main rock
    parts.append(box(-0.4, 0.0, -0.3, 0.35, 0.5, 0.4))
    # Side rock 1
    parts.append(box(0.2, 0.0, -0.15, 0.6, 0.35, 0.3))
    # Side rock 2
    parts.append(box(-0.6, 0.0, 0.0, -0.2, 0.3, 0.35))
    # Top rock
    parts.append(box(-0.2, 0.4, -0.15, 0.25, 0.7, 0.2))
    return make_glb(parts, (0.5, 0.5, 0.52, 1.0), "Stone")


def main():
    out_dir = "spikes/godot-gdext/assets/models"
    os.makedirs(out_dir, exist_ok=True)

    models = {
        "peasant.glb": generate_peasant(),
        "wolf.glb": generate_wolf(),
        "tree.glb": generate_tree(),
        "stone.glb": generate_stone(),
    }

    for name, data in models.items():
        path = os.path.join(out_dir, name)
        with open(path, "wb") as f:
            f.write(data)
        print(f"Wrote {path} ({len(data)} bytes)")

    # Write CREDITS.md
    credits_path = os.path.join(out_dir, "CREDITS.md")
    with open(credits_path, "w") as f:
        f.write("# Asset Credits\n\n")
        f.write("These are procedural fallback models generated by `scripts/asset-fallback/generate_glb.py`.\n")
        f.write("They are dedicated to the public domain (CC0).\n\n")
        f.write("| Model | Source | License |\n")
        f.write("|-------|--------|---------|\n")
        f.write("| peasant.glb | Procedurally generated by 20KBC project | CC0 |\n")
        f.write("| wolf.glb | Procedurally generated by 20KBC project | CC0 |\n")
        f.write("| tree.glb | Procedurally generated by 20KBC project | CC0 |\n")
        f.write("| stone.glb | Procedurally generated by 20KBC project | CC0 |\n")
    print(f"Wrote {credits_path}")


if __name__ == "__main__":
    main()
