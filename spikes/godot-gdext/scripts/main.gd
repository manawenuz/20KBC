extends Node3D

# GDExtension types resolve after extension loads — refs are untyped on purpose.
@onready var sim = $SimBridge
@onready var selection = $SelectionManager
@onready var units_parent: Node3D = $Units
@onready var resources_parent: Node3D = $Resources
@onready var gaia_parent: Node3D = $Gaia
@onready var camera: Camera3D = $RtsCameraController
@onready var sel_count_label: Label = $CanvasLayer/GameHud/SelectionCountLabel
@onready var box_selector = $CanvasLayer/BoxSelector
@onready var portrait = $CanvasLayer/UnitPortrait

const REPLAY_PATH := "user://replay.bin"
const PLAYER_ID := 0
const CLICK_PICK_RADIUS: float = 1.5
const RESOURCE_HIT_RADIUS: float = 2.0
const HOSTILE_HIT_RADIUS: float = 1.5

# id-keyed dictionaries. Stable across the sim's lifetime.
var unit_nodes: Dictionary = {}
var gaia_nodes: Dictionary = {}
var resource_nodes: Dictionary = {}

func _ready() -> void:
    get_tree().set_auto_accept_quit(false)
    if box_selector != null:
        box_selector.connect("selection_box", Callable(self, "_on_selection_box"))

func _notification(what: int) -> void:
    if what == NOTIFICATION_WM_CLOSE_REQUEST:
        if sim != null:
            sim.save_replay(REPLAY_PATH)
            print("Replay saved to ", REPLAY_PATH)
        get_tree().quit()

# --- Per-frame sync ------------------------------------------------------

func _physics_process(_delta: float) -> void:
    if sim == null:
        return
    _sync_units()
    _sync_gaia()
    _sync_resources()
    _push_unit_hp()
    _refresh_portrait()
    if sel_count_label != null:
        sel_count_label.text = "Selected: %d" % selection.count()

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
    var to_remove: Array = []
    for rid in resource_nodes.keys():
        if not live_ids.has(rid):
            to_remove.append(rid)
    for rid in to_remove:
        (resource_nodes[rid] as Node).queue_free()
        resource_nodes.erase(rid)

func _push_unit_hp() -> void:
    # Drive UnitNode.set_hp() so the combat-fx code (hit flash, damage numbers)
    # has accurate per-frame HP deltas to react to.
    for uid in unit_nodes.keys():
        var hp: float = sim.get_unit_hp(int(uid))
        if hp >= 0.0:
            (unit_nodes[uid]).call("set_hp", hp)

func _refresh_portrait() -> void:
    if portrait == null:
        return
    var sel: Array = selection.get_all()
    if sel.is_empty():
        portrait.hide_panel()
        return
    if sel.size() == 1:
        var uid: int = int(sel[0])
        var stats: Array = sim.get_unit_stats(uid)
        if stats.size() == 4:
            portrait.show_single(stats[0], stats[1], stats[2], stats[3])
        return
    # Multi-selection: aggregate HP.
    var total_hp: float = 0.0
    var total_max: float = 0.0
    for raw in sel:
        var s: Array = sim.get_unit_stats(int(raw))
        if s.size() == 4:
            total_hp += s[0]
            total_max += s[1]
    portrait.show_multi(sel.size(), total_hp, total_max)

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
        if event.keycode == KEY_G:
            sim.spawn_workers(40)

func _ray_hit_ground(screen_pos: Vector2) -> Vector3:
    var origin: Vector3 = camera.project_ray_origin(screen_pos)
    var dir: Vector3 = camera.project_ray_normal(screen_pos)
    if dir.y == 0.0:
        return Vector3.ZERO
    var t: float = -origin.y / dir.y
    return origin + dir * t

func _clear_selection_visuals() -> void:
    for pid in selection.get_all():
        if unit_nodes.has(int(pid)):
            unit_nodes[int(pid)].call("set_selected", false)
    selection.clear()

func _add_to_selection(uid: int) -> void:
    selection.add(uid)
    if unit_nodes.has(uid):
        unit_nodes[uid].call("set_selected", true)

func _handle_left_click(screen_pos: Vector2) -> void:
    # Single-click select. BoxSelector handles drag separately via signal.
    var hit: Vector3 = _ray_hit_ground(screen_pos)
    var uid: int = sim.get_unit_at(hit.x, hit.z, CLICK_PICK_RADIUS)
    _clear_selection_visuals()
    if uid >= 0:
        _add_to_selection(uid)

func _on_selection_box(start: Vector2, end: Vector2) -> void:
    # Box-drag select: project both corners to world, then query sim.
    var a: Vector3 = _ray_hit_ground(start)
    var b: Vector3 = _ray_hit_ground(end)
    var min_x: float = minf(a.x, b.x)
    var max_x: float = maxf(a.x, b.x)
    var min_z: float = minf(a.z, b.z)
    var max_z: float = maxf(a.z, b.z)
    var ids: Array = sim.get_units_in_rect(min_x, min_z, max_x, max_z)
    _clear_selection_visuals()
    for raw in ids:
        _add_to_selection(int(raw))

func _handle_right_click(screen_pos: Vector2) -> void:
    var hit: Vector3 = _ray_hit_ground(screen_pos)
    var sel: Array = selection.get_all()
    if sel.is_empty():
        return

    # Attack priority: hostile unit > resource node > ground.
    var hostile_id: int = sim.get_hostile_at(hit.x, hit.z, PLAYER_ID, HOSTILE_HIT_RADIUS)
    if hostile_id >= 0:
        for raw_uid in sel:
            sim.issue_attack_order(int(raw_uid), hostile_id)
        return

    var resource_id: int = _resource_at(Vector2(hit.x, hit.z))
    if resource_id >= 0:
        for raw_uid in sel:
            sim.issue_gather_order(int(raw_uid), resource_id)
        return

    # Ground click → formation move so the group doesn't clump.
    sim.issue_formation_move(sel, hit.x, hit.z)

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
