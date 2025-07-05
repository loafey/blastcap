using Godot;
using System.Collections.Generic;

public partial class Actor : Node3D {
    private string _actorName;
    private int _maxHealth;
    private int _health;
    public int MaxHealth {
        get => this._maxHealth; set {
            this._maxHealth = value;
            this.UpdateLabel();
        }
    }
    public int Health {
        get => this._health; set {
            this._health = value;
            this.UpdateLabel();
        }
    }

    [Export]
    public Label3D ActorLabel;

    private void UpdateLabel() {
        this.ActorLabel.Text = $"{this._actorName}\n{this._health}/{this._maxHealth}";
    }

    public string ActorName {
        get => this._actorName;
        set {
            this._actorName = value;
            this.UpdateLabel();
        }
    }


    public List<string> Abilities = [];
    private List<Vector3I> _walkGoals = [];
    private Vector3 _curPos = new();
    private int _posCount = 0;

    public override void _Ready() {
        base._Ready();
        this.UpdateLabel();
    }

    public override void _PhysicsProcess(double delta) {
        base._PhysicsProcess(delta);
        if (this._walkGoals.Count != 0) {
            var movement = Engine.PhysicsTicksPerSecond / (int)Constants.TILES_PER_SECOND;
            this._posCount += 1;
            this.Position = this._curPos.Lerp(
                this._walkGoals[0],
                (float)this._posCount / movement
            );
            if (this._posCount > movement) {
                this._curPos = this._walkGoals[0];
                this._walkGoals.RemoveAt(0);
                this._posCount = 0;
            }
        } else {
            this.Position = this.Position.Round();
        }
    }

    public void MoveTo(List<Vector3I> goal) {
        if (goal.Count > 0) {
            this._curPos = goal[0];
            goal.RemoveAt(0);
            this._posCount = 0;
            this._walkGoals = goal;
        }
    }
}
