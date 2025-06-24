@tool
extends Button

var popup

var label
var scrollArea: ScrollContainer
var exitButton: Button
var textContent = ""

var process = null
var processOut_io: FileAccess
var threadOut


func _on_press():
    textContent = ""
    # process = OS.execute_with_pipe("curl", ["https://loafey.se"])
    # processOut = OS.execute_with_pipe("sh", ["-c", "\"sleep 1s && cd .. && ls\""])
    # process = OS.execute_with_pipe("bash", ["-c", "\"sleep 1s && echo yo\""])
    process = OS.execute_with_pipe("bash", ["-c", "cargo build 2>&1"])

    processOut_io = process['stdio']
    threadOut = Thread.new()
    threadOut.start(read_processOut_output)

    # create popup

    label = Label.new()
    label.text = ""
    label.set_anchors_preset(Control.PRESET_FULL_RECT)

    scrollArea = ScrollContainer.new()
    scrollArea.set_anchors_preset(Control.PRESET_FULL_RECT)
    scrollArea.add_child(label)
    # scrollArea.get_v_scroll_bar().connect("changed", scrollbar_change())

    exitButton = Button.new()
    exitButton.set_anchors_and_offsets_preset(Control.LayoutPreset.PRESET_TOP_RIGHT)
    exitButton.position.x -= 32
    exitButton.text = "Exit"
    exitButton.connect("pressed", exit_clicked)

    popup = Window.new()
    popup.initial_position = Window.WINDOW_INITIAL_POSITION_CENTER_PRIMARY_SCREEN
    popup.size = Vector2i(800, 400)
    popup.exclusive = true
    popup.borderless = false
    popup.unfocusable = false
    popup.add_child(scrollArea)
    popup.add_child(exitButton)

    EditorInterface.popup_dialog(popup)


func exit_clicked():
    if popup != null:
        popup.queue_free()
        textContent = ""
        popup = null

func scrollbar_change():
    scrollArea.scroll_vertical = scrollArea.get_v_scroll_bar().max_value


func read_processOut_output():
    while processOut_io.get_error() == OK:
        var line = processOut_io.get_line()
        textContent += line + "\n"
    

func _process(delta: float) -> void:
    if label != null:
        label.text = textContent

func _ready():
    self.pressed.connect(_on_press)