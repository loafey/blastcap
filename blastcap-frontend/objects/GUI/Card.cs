using Godot;
using System;

public partial class Card : Control {
    private ulong cardId;
    private Vector2 currentOffset;
    private Vector2 targetOffset;
    private Vector2 currentScale = new(1, 1);
    private Vector2 targetScale = new(1, 1);

    [Export]
    private Label cardTitleLabel;
    [Export]
    private Panel cardPanel;

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
