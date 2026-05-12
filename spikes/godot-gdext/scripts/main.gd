extends Node3D

# GDExtension types resolve after extension loads — refs are untyped on purpose.
@onready var sim = $SimBridge
@onready var selection = $SelectionManager
@onready var units_parent: Node3D = $Units
@onready var resources_parent: Node3D = $Resources
@onready var gaia_parent: Node3D = $Gaia
@onready var camera: Camera3D = $RtsCameraController
@onready var sel_count_label: Label = $CanvasLayer/GameHud/SelectionCountLabel

const REPLAY_PATH := "user://replay.bin"

func _ready() -> void:
    # Save replay on window close.
    get_tree().set_auto_accept_quit(false)

func _notification(what: int) -> void:
    if what == NOTIFICATION_WM_CLOSE_REQUEST:
        if sim != null:
            sim.save_replay(REPLAY_PATH)
            print("Replay saved to ", REPLAY_PATH)
        get_tree().quit()

# id-keyed dictionaries. Stable across the sim's lifetime.
var unit_nodes: Dictionary = {}   # int unit_id -> UnitNode
var gaia_nodes: Dictionary = {}   # int gaia_id -> GaiaNode (best-effort: index-keyed for now)
var resource_nodes: Dictionary = {}  # int node_id -> ResourceNode

# Click selection radius in world units.
const CLICK_PICK_RADIUS: float = 1.5
# Treat right-click within this distance of a resource node as "gather".
const RESOURCE_HIT_RADIUS: float = 2.0

func _physics_process(_delta: float) -> void:
    if sim == null:
        return
    _sync_units()
    _sync_gaia()
    _sync_resources()
    if sel_count_label != null:
        sel_count_label.text = "Selected: %d" % selection.count()

# --- Sync helpers ---------------------------------------------------------

func _sync_units() -> void:
    var ids: Array = sim.get_unit_ids()
    var positions: Array = sim.get_unit_positions()
    var live_ids := {}
    for i in range(ids.size()):
        var uid: int = int(ids[i])
        live_ids[uid] = true
        var v: Vector2 = positions[i]
        if not unit_nodes.has(uid):
            var node = ClassDB.instantiate("UnitNode")
            if node == null:
                continue
            node.set("unit_id", uid)
            units_parent.add_child(node)
            unit_nodes[uid] = node
        (unit_nodes[uid] as Node3D).position = Vector3(v.x, 0.0, v.y)
    # Cull nodes for units that are no longer alive.
    var to_remove: Array = []
    for uid in unit_nodes.keys():
        if not live_ids.has(uid):
            to_remove.append(uid)
    for uid in to_remove:
        (unit_nodes[uid] as Node).queue_free()
        unit_nodes.erase(uid)
        selection.remove(uid)

func _sync_gaia() -> void:
    var positions: Array = sim.get_gaia_positions()
    # Spawn-on-demand; we don't have stable gaia ids exported yet, so key by index.
    for i in range(positions.size()):
        if not gaia_nodes.has(i):
            var node = ClassDB.instantiate("GaiaNode")
            if node == null:
                continue
            node.set("gaia_id", i)
            gaia_parent.add_child(node)
            gaia_nodes[i] = node
        var v: Vector2 = positions[i]
        (gaia_nodes[i] as Node3D).position = Vector3(v.x, 0.0, v.y)

func _sync_resources() -> void:
    var nodes: Array = sim.get_resource_nodes()
    var kinds: Array = sim.get_resource_kinds()
    var live_ids := {}
    for i in range(nodes.size()):
        var v: Vector3 = nodes[i]
        var rid: int = int(v.x)
        var kind: int = int(kinds[i])
        live_ids[rid] = true
        if not resource_nodes.has(rid):
            var rn = ClassDB.instantiate("ResourceNode")
            if rn == null:
                continue
            rn.set("kind", kind)
            rn.set("node_id", rid)
            resources_parent.add_child(rn)
            resource_nodes[rid] = rn
        (resource_nodes[rid] as Node3D).position = Vector3(v.y, 0.0, v.z)
    # Remove depleted nodes.
    var to_remove: Array = []
    for rid in resource_nodes.keys():
        if not live_ids.has(rid):
            to_remove.append(rid)
    for rid in to_remove:
        (resource_nodes[rid] as Node).queue_free()
        resource_nodes.erase(rid)

# --- Input ---------------------------------------------------------------

func _input(event: InputEvent) -> void:
    if sim == null:
        return
    if event is InputEventMouseButton and event.pressed:
        if event.button_index == MOUSE_BUTTON_LEFT:
            _handle_left_click(event.position)
        elif event.button_index == MOUSE_BUTTON_RIGHT:
            _handle_right_click(event.position)
    elif event is InputEventKey and event.pressed and not event.echo:
        # Stress-test hotkey: G spawns 40 extra workers.
        if event.keycode == KEY_G:
            sim.spawn_workers(40)

func _ray_hit_ground(screen_pos: Vector2) -> Vector3:
    var origin: Vector3 = camera.project_ray_origin(screen_pos)
    var dir: Vector3 = camera.project_ray_normal(screen_pos)
    if dir.y == 0.0:
        return Vector3.ZERO
    var t: float = -origin.y / dir.y
    return origin + dir * t

func _handle_left_click(screen_pos: Vector2) -> void:
    var hit: Vector3 = _ray_hit_ground(screen_pos)
    var uid: int = sim.get_unit_at(hit.x, hit.z, CLICK_PICK_RADIUS)
    # Clear previous selection visuals.
    var prev: Array = selection.get_all()
    for pid in prev:
        if unit_nodes.has(int(pid)):
            unit_nodes[int(pid)].set_selected(false)
    selection.clear()
    if uid >= 0:
        selection.add(uid)
        if unit_nodes.has(uid):
            unit_nodes[uid].set_selected(true)

func _handle_right_click(screen_pos: Vector2) -> void:
    var hit: Vector3 = _ray_hit_ground(screen_pos)
    var sel: Array = selection.get_all()
    if sel.is_empty():
        return
    # Did we click on a resource?
    var hit_resource_id: int = _resource_at(Vector2(hit.x, hit.z))
    for raw_uid in sel:
        var uid: int = int(raw_uid)
        if hit_resource_id >= 0:
            sim.issue_gather_order(uid, hit_resource_id)
        else:
            sim.issue_move_order(uid, hit.x, hit.z)

func _resource_at(world_xz: Vector2) -> int:
    var nodes: Array = sim.get_resource_nodes()
    var best_id: int = -1
    var best_d: float = RESOURCE_HIT_RADIUS
    for v in nodes:
        var rid: int = int((v as Vector3).x)
        var rp := Vector2((v as Vector3).y, (v as Vector3).z)
        var d: float = rp.distance_to(world_xz)
        if d < best_d:
            best_d = d
            best_id = rid
    return best_id
