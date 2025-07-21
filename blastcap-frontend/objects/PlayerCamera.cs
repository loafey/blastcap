using Godot;
using System;

public partial class PlayerCamera : Node3D {
    private float _cameraSpeed = 5.0f;
    private float _cameraRotationSpeed = 1.4f;
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
    [Export]
    public Label RTTLabel;
    [Export]
    public ProgressBar MovementBar;
    [Export]
    public Label MovementLabel;
    public Actor MyActor;

    public (uint, uint) Movement {
        set {
            this.MovementBar.Value = value.Item1;
            this.MovementBar.MaxValue = value.Item2;
            this.MovementLabel.Text = $"{value.Item1}/{value.Item2}";
        }
    }

    public (int, ulong) RTT {
        set => this.RTTLabel.Text = $"RTT: {value.Item1}ms ({value.Item2})";
    }

    private bool _myTurn = false;
    public bool MyTurn {
        get => this._myTurn;
        set {
            this._myTurn = value;
            if (value) {
                this.EnableActions();
            } else {
                this.DisableActions();
            }
        }
    }

    public Action EndTurnPressed {
        set => this.EndTurnButton.Pressed += () => {
            this.EndTurnButton.ReleaseFocus();
            value();
        };
    }

    public string CurrentAbility {
        set => this.CurrentAbilityLabel.Text = value == null
            ? ""
            : $"Current ability: {value}";
    }

    public void DisableActions() {
        foreach (var child in this.AbilitiesGrid.GetChildren()) {
            if (child is Button button) {
                button.Disabled = true;
            }
        }
        this.EndTurnButton.Disabled = true;
    }
    public void EnableActions() {
        foreach (var child in this.AbilitiesGrid.GetChildren()) {
            if (child is Button button) {
                button.Disabled = false;
            }
        }
        this.EndTurnButton.Disabled = false;
    }

    public void AddAbilityButton(string name, string tooltip, Action callback) {
        var button = new Button {
            Text = name,
            TooltipText = tooltip,
            KeepPressedOutside = false
        };
        button.Pressed += () => {
            button.ReleaseFocus();
            callback();
        };
        this.AbilitiesGrid.AddChild(button);
    }

    public override void _Ready() {
        base._Ready();
        var rot = this.Camera.Rotation;
        rot.X = -0.9f;
        this.Camera.Rotation = rot;
    }

    private void RotateCam(Vector2 rotation) {
        this.BoomArm.RotateY(-rotation.X);
        this.Camera.RotateX(-rotation.Y);
        var rot = this.Camera.Rotation;
        rot.X = this.Camera.Projection == Camera3D.ProjectionType.Orthogonal
            ? Mathf.Clamp(rot.X, -Mathf.Pi / 2, MathF.PI / 4)
            : Mathf.Clamp(rot.X, -Mathf.Pi / 2, Mathf.Pi / 4);
        this.Camera.Rotation = rot;
    }

    public override void _Input(InputEvent @event) {
        base._Input(@event);
        if (this._cameraLock && @event is InputEventMouseMotion ev) {
            this.RotateCam(ev.Relative * this._cameraBoomSpeed);
        }
    }

    public override void _Process(double delta) {
        base._Process(@delta);
        var newPos = this.Position;
        var sin = (float)delta * this._cameraSpeed * Mathf.Sin(this.BoomArm.Rotation.Y);
        var cos = (float)delta * this._cameraSpeed * Mathf.Cos(this.BoomArm.Rotation.Y);
        if (Input.IsActionJustPressed("camera_ortho_switch")) {
            if (this.Camera.Projection == Camera3D.ProjectionType.Perspective) {
                this.Camera.Projection = Camera3D.ProjectionType.Orthogonal;
                this.Camera.Size = 20f;
                var rot = this.Camera.Rotation;
                // rot.X = Mathf.Clamp(rot.X, -Mathf.Pi / 2, -0.9f);
                this.Camera.Rotation = rot;
            } else {
                this.Camera.Projection = Camera3D.ProjectionType.Perspective;
                this.Camera.Size = 20f;
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
        if (Input.IsActionPressed("camera_float_up")) {
            newPos.Y += (float)delta * this._cameraSpeed;
        } else if (Input.IsActionPressed("camera_float_down")) {
            newPos.Y -= (float)delta * this._cameraSpeed;
        }

        newPos.Y = Mathf.Clamp(newPos.Y, -3f, 40f);
        this.Position = newPos;

        if (Input.IsActionPressed("camera_rotate_lock")) {
            this._cameraLock = true;
            Input.MouseMode = Input.MouseModeEnum.Captured;
        } else {
            this._cameraLock = false;
            Input.MouseMode = Input.MouseModeEnum.Visible;
        }


        if (Input.IsActionPressed("camera_rotate_left")) {
            this.RotateCam(new Vector2(-(float)delta * this._cameraRotationSpeed, 0));
        } else if (Input.IsActionPressed("camera_rotate_right")) {
            this.RotateCam(new Vector2((float)delta * this._cameraRotationSpeed, 0));
        }

        if (Input.IsActionPressed("camera_pan_up")) {
            this.RotateCam(new Vector2(0, -(float)delta * this._cameraRotationSpeed));
        } else if (Input.IsActionPressed("camera_pan_down")) {
            this.RotateCam(new Vector2(0, (float)delta * this._cameraRotationSpeed));
        }
    }

    public void DisplayTinyPopup(string text) {
        var scene = this.TinyPopupScene.Instantiate<TinyPopup>();
        scene.Visible = false;
        scene.Text = text;
        this.TinyPopupHolder.AddChild(scene);
    }
}
