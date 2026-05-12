extends Node3D

# Use untyped refs — GDExtension types resolve after extension loads.
@onready var sim = $SimBridge
@onready var units_parent: Node3D = $Units

var unit_nodes: Dictionary = {}

func _physics_process(_delta: float) -> void:
    if sim == null:
        return
    _sync_units()

func _sync_units() -> void:
    var positions: Array = sim.get_unit_positions()
    var count: int = positions.size()

    for i in range(count):
        if not unit_nodes.has(i):
            # UnitNode is a GDExtension class — instantiate by name.
            var unit_node = ClassDB.instantiate("UnitNode")
            if unit_node == null:
                continue
            unit_node.set("unit_id", i)
            units_parent.add_child(unit_node)
            unit_nodes[i] = unit_node

    for i in range(count):
        if unit_nodes.has(i):
            var v: Vector2 = positions[i]
            (unit_nodes[i] as Node3D).position = Vector3(v.x, 0.0, v.y)

func _input(event: InputEvent) -> void:
    if sim == null:
        return
    if event is InputEventMouseButton \
            and event.button_index == MOUSE_BUTTON_RIGHT \
            and event.pressed:
        var camera: Camera3D = $RtsCameraController
        var ray_origin: Vector3 = camera.project_ray_origin(event.position)
        var ray_dir: Vector3 = camera.project_ray_normal(event.position)
        if ray_dir.y != 0.0:
            var t: float = -ray_origin.y / ray_dir.y
            var hit: Vector3 = ray_origin + ray_dir * t
            sim.issue_move_order(0, hit.x, hit.z)
