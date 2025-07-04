using Godot;
using System;

public partial class PlayerCamera : Node3D {
    private float _cameraSpeed = 5.0f;
    private float _cameraBoomSpeed = 0.005f;
    private bool _cameraLock = false;
    private Node3D _boomArm;
    private Camera3D _camera;
    public Camera3D Camera { get => _camera; }
    private Control _tinyPopupHolder;
    private GridContainer _abilitiesGrid;
    private PackedScene _tinyPopupScene;
    public Actor MyActor;

    private bool _myTurn = false;
    public bool MyTurn {
        get => _myTurn;
        set {
            _myTurn = value;
        }
    }

    public void AddAbilityButton(string name, string tooltip, Action callback) {
        var button = new Button();
        button.Text = name;
        button.TooltipText = tooltip;
        button.Pressed += callback;
        _abilitiesGrid.AddChild(button);
    }

    public override void _Ready() {
        base._Ready();
        _tinyPopupHolder = GetNode<Control>("CanvasLayer/TinyPopupHolder");
        _boomArm = GetNode<Node3D>("BoomArm");
        _camera = GetNode<Camera3D>("BoomArm/Camera");
        _tinyPopupScene = GD.Load<PackedScene>("uid://bp7yq4iqifwrh");
        var rot = _camera.Rotation;
        rot.X = -0.9f;
        _camera.Rotation = rot;
        _abilitiesGrid = GetNode<GridContainer>("CanvasLayer/Panel/HBoxContainer/AbilitiesGrid");
    }

    private void RotateCam(Vector2 rotation) {
        _boomArm.RotateY(-rotation.X);
        _camera.RotateX(-rotation.Y);
        var rot = _camera.Rotation;
        if (_camera.Projection == Camera3D.ProjectionType.Orthogonal)
            rot.X = Mathf.Clamp(rot.X, -Mathf.Pi / 2, -0.9f);
        else
            rot.X = Mathf.Clamp(rot.X, -Mathf.Pi / 2, -0.1f);
        _camera.Rotation = rot;
    }

    public override void _Input(InputEvent @event) {
        base._Input(@event);
        if (_cameraLock && @event is InputEventMouseMotion ev)
            RotateCam(-ev.Relative * _cameraBoomSpeed);
    }

    public override void _Process(double delta) {
        base._Process(@delta);
        var newPos = Position;
        var sin = (float)delta * _cameraSpeed * Mathf.Sin(_boomArm.Rotation.Y);
        var cos = (float)delta * _cameraSpeed * Mathf.Cos(_boomArm.Rotation.Y);
        if (Input.IsActionJustPressed("camera_ortho_switch")) {
            if (_camera.Projection == Camera3D.ProjectionType.Perspective) {
                _camera.Projection = Camera3D.ProjectionType.Orthogonal;
                _camera.Size = 20f;
                var rot = _camera.Rotation;
                rot.X = Mathf.Clamp(rot.X, -Mathf.Pi / 2, -0.9f);
                _camera.Rotation = rot;
            } else {
                _camera.Projection = Camera3D.ProjectionType.Perspective;
                _camera.Size = 20f;
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


        if (Input.IsActionPressed("camera_rotate_left"))
            RotateCam(new Vector2(-(float)delta, 0));
        else if (Input.IsActionPressed("camera_rotate_right"))
            RotateCam(new Vector2((float)delta, 0));

        if (Input.IsActionPressed("camera_pan_up"))
            RotateCam(new Vector2(0, -(float)delta));
        else if (Input.IsActionPressed("camera_pan_down"))
            RotateCam(new Vector2(0, (float)delta));
    }

    public void DisplayTinyPopup(String text) {
        var scene = _tinyPopupScene.Instantiate<TinyPopup>();
        scene.Visible = false;
        scene.Text = text;
        _tinyPopupHolder.AddChild(scene);
    }
}
