@tool
extends CanvasLayer
class_name Map


# child_scene specifies the target that this scene
# will transition into when entered.
@export_file var child_scene: String
# parent_scene specifies the target parent that
# this scene will transition to when exited.
@export_file var parent_scene: String


# interactable_area is the child interaction zone for this node.
#
# When an interaction occurs within this area, the node should act.
@onready var interactable_area: Area2D = find_child("Area2D")


func _ready() -> void:
	# if we have a child_scene and an interactable area, listen to events
	if child_scene && interactable_area:
		interactable_area.connect("input_event", _on_area_2d_input_event)


func _get_configuration_warnings() -> PackedStringArray:
	var warnings = []

	if child_scene && !interactable_area:
		warnings.append("Map's with a child_scene must have an Area2D")

	# Returning an empty array means "no warning".
	return warnings


func _on_area_2d_input_event(_viewport: Node, event: InputEvent, _shape_idx: int) -> void:
	if event.is_action_pressed("zoom_in"):
		open_child_map()


func _input(event: InputEvent) -> void:
	if event.is_action("zoom_out"):
		close_child_map()



func close_child_map() -> void:
	if !parent_scene:
		return
	
	Messenger.transition_scene.emit(load(parent_scene))


func open_child_map() -> void:
	if !child_scene:
		return
	
	Messenger.transition_scene.emit(load(child_scene))
