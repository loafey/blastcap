using Godot;
using System;

public partial class PlayerCamera : Node3D
{
    private float _cameraSpeed = 5.0f;
    private float _cameraBoomSpeed = 0.005f;
    private bool _cameraLock = false;
    private Node3D _boomArm;
    private Camera3D _camera;
    public Camera3D Camera { get => _camera; }
    private Control _tinyPopupHolder;
    private PackedScene _tinyPopupScene;

    private bool _myTurn = false;
    public bool MyTurn
    {
        get => _myTurn;
        set
        {
            _myTurn = value;
        }
    }

    public override void _Ready()
    {
        base._Ready();
        _tinyPopupHolder = GetNode<Control>("CanvasLayer/TinyPopupHolder");
        _boomArm = GetNode<Node3D>("BoomArm");
        _camera = GetNode<Camera3D>("BoomArm/Camera");
        _tinyPopupScene = GD.Load<PackedScene>("uid://bp7yq4iqifwrh");
    }

    public override void _Input(InputEvent @event)
    {
        base._Input(@event);
        if (_cameraLock && @event is InputEventMouseMotion ev)
        {
            _boomArm.RotateY(-ev.Relative.X * _cameraBoomSpeed);
            _camera.RotateX(-ev.Relative.Y * _cameraBoomSpeed);
            var rot = _camera.Rotation;
            if (_camera.Projection == Camera3D.ProjectionType.Orthogonal)
                rot.X = Mathf.Clamp(rot.X, -0.9f, -0.9f);
            else
                rot.X = Mathf.Clamp(rot.X, -Mathf.Pi / 2, -0.1f);
            _camera.Rotation = rot;
        }
    }

    public override void _Process(double delta)
    {
        base._Process(@delta);
        var newPos = Position;
        var sin = (float)delta * _cameraSpeed * Mathf.Sin(_boomArm.Rotation.Y);
        var cos = (float)delta * _cameraSpeed * Mathf.Cos(_boomArm.Rotation.Y);
        if (Input.IsActionJustPressed("camera_ortho_switch"))
        {
            if (_camera.Projection == Camera3D.ProjectionType.Perspective)
            {
                _camera.Projection = Camera3D.ProjectionType.Orthogonal;
                _camera.Size = 20f;
                var rot = _camera.Rotation;
                rot.X = Mathf.Clamp(rot.X, -0.9f, -0.9f);
                _camera.Rotation = rot;
            }
            else
            {
                _camera.Projection = Camera3D.ProjectionType.Perspective;
                _camera.Size = 20f;
            }
        }

        if (Input.IsActionPressed("camera_left"))
        {
            newPos.X -= cos;
            newPos.Z += sin;
        }
        else if (Input.IsActionPressed("camera_right"))
        {
            newPos.X += cos;
            newPos.Z -= sin;
        }
        if (Input.IsActionPressed("camera_up"))
        {
            newPos.Z -= cos;
            newPos.X -= sin;
        }
        else if (Input.IsActionPressed("camera_down"))
        {
            newPos.Z += cos;
            newPos.X += sin;
        }
        Position = newPos;

        if (Input.IsActionPressed("camera_rotate_lock"))
        {
            _cameraLock = true;
            Input.MouseMode = Input.MouseModeEnum.Captured;
        }
        else
        {
            _cameraLock = false;
            Input.MouseMode = Input.MouseModeEnum.Visible;
        }
    }

    public void DisplayTinyPopup(String text)
    {
        var scene = _tinyPopupScene.Instantiate<TinyPopup>();
        scene.Visible = false;
        scene.Text = text;
        _tinyPopupHolder.AddChild(scene);
    }
}
