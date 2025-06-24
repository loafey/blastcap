@tool
extends EditorPlugin

const buttonScene = preload("res://addons/commandrunner/CommandRunner.tscn")
var instance;

func _enter_tree() -> void:
	# Initialization of the plugin goes here.
	instance = buttonScene.instantiate();
	add_control_to_container(CONTAINER_TOOLBAR, instance);
	
	pass


func _exit_tree() -> void:
	# Clean-up of the plugin goes here.
	remove_control_from_container(CONTAINER_TOOLBAR, instance)
	pass
