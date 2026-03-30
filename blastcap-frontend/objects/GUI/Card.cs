using Godot;
using System;

public partial class Card : Control {
    private ulong cardId;
    private Vector2 currentOffset;
    private Vector2 targetOffset;
    private Vector2 currentScale = new(1, 1);
    private Vector2 targetScale = new(1, 1);
    private bool howering;

    public bool Disabled;

    [Export]
    private Label cardTitleLabel;
    [Export]
    private Panel cardPanel;

    public Action OnClick;

    public ulong CardId {
        get => this.cardId;
        set {
            this.cardId = value;
            var data = Data.Cards[this.cardId];
            this.cardTitleLabel.Text = data.Name;
        }
    }

    [Export]
    public Vector2 HoverOffset = new(0, -4);
    [Export]
    public Vector2 HoverScale = new(1.1f, 1.1f);

    public override void _Ready() {
        base._Ready();

        MouseEntered += this.MouseEnter;
        MouseExited += this.MouseLeave;
    }

    public override void _Input(InputEvent @event) {
        base._Input(@event);
        if (@event.IsActionPressed("ui_select") && this.howering && !this.Disabled) {
            this.OnClick?.Invoke();
        }
    }


    public override void _PhysicsProcess(double delta) {
        base._PhysicsProcess(delta);
        this.currentOffset = this.currentOffset.Lerp(this.targetOffset, (float)delta * 10);
        this.currentScale = this.currentScale.Lerp(this.targetScale, (float)delta * 10);
        this.Scale = this.currentScale;
        this.cardPanel.Position = this.currentOffset;
    }


    private void MouseEnter() {
        if (this.Disabled) { return; }
        this.targetOffset = this.HoverOffset;
        this.targetScale = this.HoverScale;
        this.howering = true;
    }

    private void MouseLeave() {
        this.targetOffset = new();
        this.targetScale = new(1, 1);
        this.howering = false;
    }
}
