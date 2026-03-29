using Godot;
using System;

public partial class Loading : Node2D {
    private readonly Random random = new();
    private NetworkManager nw;

    [Export]
    private Label loadingLabel;

    [Export]
    private Label tidbitLabel;

    [Export]
    private TextureRect spinner;

    [Export]
    private Timer loadingTimer;
    private int loadingTick = 1;

    [Export]
    private Timer tidbitTimer;

    private string[] tidbits = [
        "The grass could be greener...",
        "moo..."
    ];

    private int currentIndex = -1;
    private void RandomTidbit() {
        var index = -1;
        while (true) {
            index = this.random.Next(0, this.tidbits.Length);
            if (index == this.currentIndex) { continue; }
            this.currentIndex = index;
            break;
        }
        this.tidbitLabel.Text = this.tidbits[index];
    }

    public override void _Ready() {
        base._Ready();
        this.nw = this.GetNode<NetworkManager>("/root/NetworkManager");
        this.RandomTidbit();
        this.loadingTimer.Timeout += () => {
            this.loadingTick = (this.loadingTick + 1) % 4;
            this.loadingLabel.Text = $"Loading{new string('.', this.loadingTick)}";

            // TODO: remove this once the actual loading has been implemented
            this.GetTree().ChangeSceneToFile("uid://bo5tvenb8rnc8");
        };
        this.tidbitTimer.Timeout += this.RandomTidbit;
    }

    public override void _PhysicsProcess(double delta) {
        base._PhysicsProcess(delta);
        this.spinner.Rotation -= (float)delta;
    }

}
