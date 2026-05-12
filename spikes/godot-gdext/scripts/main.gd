extends Node3D

@onready var sim: SimBridge = $SimBridge
@onready var units_parent: Node3D = $Units

## Maps unit_id (int) → UnitNode instance.
var unit_nodes: Dictionary = {}

## Preload nothing — UnitNode is a GDExtension class registered by Rust.
## We instantiate it directly by class name.

func _physics_process(_delta: float) -> void:
    _sync_units()

## Keep the visual unit count in sync with the simulation.
## For MVP this simply ensures a UnitNode exists for each living unit index.
func _sync_units() -> void:
    var positions: Array = sim.get_unit_positions()
    var count: int = positions.size()

    # Spawn nodes for any new units.
    for i in range(count):
        if not unit_nodes.has(i):
            var unit_node = UnitNode.new()
            unit_node.unit_id = i
            units_parent.add_child(unit_node)
            unit_nodes[i] = unit_node

    # Update positions. positions[i] is a Vector2(x, z).
    for i in range(count):
        var v: Vector2 = positions[i]
        (unit_nodes[i] as Node3D).position = Vector3(v.x, 0.0, v.y)

func _input(event: InputEvent) -> void:
    if event is InputEventMouseButton \
            and event.button_index == MOUSE_BUTTON_RIGHT \
            and event.pressed:
        var camera: Camera3D = $RtsCameraController
        var ray_origin: Vector3 = camera.project_ray_origin(event.position)
        var ray_dir: Vector3 = camera.project_ray_normal(event.position)
        # Intersect with the y = 0 ground plane.
        if ray_dir.y != 0.0:
            var t: float = -ray_origin.y / ray_dir.y
            var hit: Vector3 = ray_origin + ray_dir * t
            # Issue move order to unit 0 for quick smoke-test.
            sim.issue_move_order(0, hit.x, hit.z)
