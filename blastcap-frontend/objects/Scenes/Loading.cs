using Godot;
using System;
using System.Buffers;

public partial class Loading : Node2D {
    private readonly Random random = new();
    private NetworkManager nw;
    private ulong loadTotal = int.MaxValue;
    private ulong loadCurrent;

    [Export]
    private Label loadingLabel;

    [Export]
    private Label tidbitLabel;

    [Export]
    private Label loadingProgress;

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

    private void UpdateProgress() {
        this.loadingProgress.Text = $"({this.loadCurrent}/{this.loadTotal})";
        if (this.loadTotal == this.loadCurrent) {
            this.GetTree().ChangeSceneToFile("uid://bo5tvenb8rnc8");
        }
    }

    public override void _Ready() {
        base._Ready();
        this.nw = this.GetNode<NetworkManager>("/root/NetworkManager");

        this.nw.Inner.OnGameLoadingTotal += (amount) => {
            this.loadTotal = amount;
            this.UpdateProgress();
        };
        this.nw.Inner.OnGameLoadingCard += (id, card) => {
            Data.Cards.Add(id, card);
            this.loadCurrent += 1;
            this.UpdateProgress();
        };
        this.nw.Inner.OnGameLoadingAbility += (id, card) => {
            Data.Abilities.Add(id, card);
            this.loadCurrent += 1;
            this.UpdateProgress();
        };

        this.nw.Inner.StartLoadingGameContent();
        this.RandomTidbit();
        this.loadingTimer.Timeout += () => {
            this.loadingTick = (this.loadingTick + 1) % 4;
            this.loadingLabel.Text = $"Loading{new string('.', this.loadingTick)}";
        };
        this.tidbitTimer.Timeout += this.RandomTidbit;
    }

    public override void _PhysicsProcess(double delta) {
        base._PhysicsProcess(delta);
        this.spinner.Rotation -= (float)delta;
    }

}
