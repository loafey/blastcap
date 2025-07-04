using Godot;
using System;

public partial class PlayerCamera : Node3D {
    private float _cameraSpeed = 5.0f;
    private float _cameraBoomSpeed = 0.005f;
    private bool _cameraLock = false;
    [Export]
    public Node3D BoomArm;
    [Export]
    public Camera3D Camera;
    [Export]
    public Control TinyPopupHolder;
    [Export]
    public GridContainer AbilitiesGrid;
    [Export]
    public PackedScene TinyPopupScene;
    [Export]
    public Button EndTurnButton;
    [Export]
    public Label CurrentAbilityLabel;
    public Actor MyActor;

    private bool _myTurn = false;
    public bool MyTurn {
        get => _myTurn;
        set {
            _myTurn = value;
            if (value) EnableActions();
            else DisableActions();
        }
    }

    public Action EndTurnPressed {
        set => EndTurnButton.Pressed += () => {
            EndTurnButton.ReleaseFocus();
            value();
        };
    }

    public string CurrentAbility {
        set {
            if (value == null) {
                CurrentAbilityLabel.Text = "";
            } else {
                CurrentAbilityLabel.Text = $"Current ability: {value}";
            }
        }
    }

    public void DisableActions() {
        foreach (Node child in AbilitiesGrid.GetChildren()) {
            if (child is Button button) {
                button.Disabled = true;
            }
        }
        EndTurnButton.Disabled = true;
    }
    public void EnableActions() {
        foreach (Node child in AbilitiesGrid.GetChildren()) {
            if (child is Button button) {
                button.Disabled = false;
            }
        }
        EndTurnButton.Disabled = false;
    }

    public void AddAbilityButton(string name, string tooltip, Action callback) {
        var button = new Button();
        button.Text = name;
        button.TooltipText = tooltip;
        button.KeepPressedOutside = false;
        button.Pressed += () => {
            button.ReleaseFocus();
            callback();
        };
        AbilitiesGrid.AddChild(button);
    }

    public override void _Ready() {
        base._Ready();
        var rot = Camera.Rotation;
        rot.X = -0.9f;
        Camera.Rotation = rot;
    }

    private void RotateCam(Vector2 rotation) {
        BoomArm.RotateY(-rotation.X);
        Camera.RotateX(-rotation.Y);
        var rot = Camera.Rotation;
        if (Camera.Projection == Camera3D.ProjectionType.Orthogonal) {
            rot.X = Mathf.Clamp(rot.X, -Mathf.Pi / 2, -0.9f);
        } else {
            rot.X = Mathf.Clamp(rot.X, -Mathf.Pi / 2, -0.1f);
        }
        Camera.Rotation = rot;
    }

    public override void _Input(InputEvent @event) {
        base._Input(@event);
        if (_cameraLock && @event is InputEventMouseMotion ev) {
            RotateCam(-ev.Relative * _cameraBoomSpeed);
        }
    }

    public override void _Process(double delta) {
        base._Process(@delta);
        var newPos = Position;
        var sin = (float)delta * _cameraSpeed * Mathf.Sin(BoomArm.Rotation.Y);
        var cos = (float)delta * _cameraSpeed * Mathf.Cos(BoomArm.Rotation.Y);
        if (Input.IsActionJustPressed("camera_ortho_switch")) {
            if (Camera.Projection == Camera3D.ProjectionType.Perspective) {
                Camera.Projection = Camera3D.ProjectionType.Orthogonal;
                Camera.Size = 20f;
                var rot = Camera.Rotation;
                rot.X = Mathf.Clamp(rot.X, -Mathf.Pi / 2, -0.9f);
                Camera.Rotation = rot;
            } else {
                Camera.Projection = Camera3D.ProjectionType.Perspective;
                Camera.Size = 20f;
            }
        }

        if (Input.IsActionPressed("camera_left")) {
            newPos.X -= cos;
            newPos.Z += sin;
        } else if (Input.IsActionPressed("camera_right")) {
            newPos.X += cos;
            newPos.Z -= sin;
        }
        if (Input.IsActionPressed("camera_up")) {
            newPos.Z -= cos;
            newPos.X -= sin;
        } else if (Input.IsActionPressed("camera_down")) {
            newPos.Z += cos;
            newPos.X += sin;
        }
        if (Input.IsActionPressed("camera_float_up"))
            newPos.Y += (float)delta * 5;
        else if (Input.IsActionPressed("camera_float_down"))
            newPos.Y -= (float)delta * 5;
        newPos.Y = Mathf.Clamp(newPos.Y, -3f, 3f);
        Position = newPos;

        if (Input.IsActionPressed("camera_rotate_lock")) {
            _cameraLock = true;
            Input.MouseMode = Input.MouseModeEnum.Captured;
        } else {
            _cameraLock = false;
            Input.MouseMode = Input.MouseModeEnum.Visible;
        }


        if (Input.IsActionPressed("camera_rotate_left")) {
            RotateCam(new Vector2(-(float)delta, 0));
        } else if (Input.IsActionPressed("camera_rotate_right")) {
            RotateCam(new Vector2((float)delta, 0));
        }

        if (Input.IsActionPressed("camera_pan_up")) {
            RotateCam(new Vector2(0, -(float)delta));
        } else if (Input.IsActionPressed("camera_pan_down")) {
            RotateCam(new Vector2(0, (float)delta));
        }
    }

    public void DisplayTinyPopup(String text) {
        var scene = TinyPopupScene.Instantiate<TinyPopup>();
        scene.Visible = false;
        scene.Text = text;
        TinyPopupHolder.AddChild(scene);
    }
}
