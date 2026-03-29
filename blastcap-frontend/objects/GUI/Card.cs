using Godot;
using System;

public partial class Card : Control {
    private string title;
    private Vector2 currentOffset;
    private Vector2 targetOffset;
    private Vector2 currentScale = new(1, 1);
    private Vector2 targetScale = new(1, 1);

    [Export]
    public string Title {
        get => this.title;
        set {
            this.title = value;
            this.SetLabel();
        }
    }

    [Export]
    private Label cardTitleLabel;
    [Export]
    private Panel cardPanel;

    [Export]
    public Vector2 HoverOffset = new(0, -4);
    [Export]
    public Vector2 HoverScale = new(1.1f, 1.1f);

    private void SetLabel() {
        this.cardTitleLabel.Text = this.Title;
    }

    public override void _Ready() {
        base._Ready();


        this.SetLabel();
        MouseEntered += this.MouseEnter;
        MouseExited += this.MouseLeave;
    }

    public override void _PhysicsProcess(double delta) {
        base._PhysicsProcess(delta);
        this.currentOffset = this.currentOffset.Lerp(this.targetOffset, (float)delta * 10);
        this.currentScale = this.currentScale.Lerp(this.targetScale, (float)delta * 10);
        this.Scale = this.currentScale;
        this.cardPanel.Position = this.currentOffset;
    }


    private void MouseEnter() {
        this.targetOffset = this.HoverOffset;
        this.targetScale = this.HoverScale;
    }

    private void MouseLeave() {
        this.targetOffset = new();
        this.targetScale = new(1, 1);
    }
}
