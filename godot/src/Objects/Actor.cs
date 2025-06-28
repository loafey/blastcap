using Godot;
using System;
using System.Collections.Generic;

public partial class Actor : Node3D
{
    private string _actorName;
    public string ActorName
    {
        get => _actorName;
        set
        {
            _actorName = value;
            var node = GetNode<Label3D>("Label3D");
            if (node != null) node.Text = value;
        }
    }

    public override void _Ready()
    {
        base._Ready();
        GetNode<Label3D>("Label3D").Text = _actorName;
        GD.Print($"ready!! {_actorName}");
    }

    private List<Vector3I> _walkGoals = new List<Vector3I>();
    private Vector3 _curPos = new Vector3();
    private int _posCount = 0;
    public override void _PhysicsProcess(double delta)
    {
        base._PhysicsProcess(delta);
        if (_walkGoals.Count != 0)
        {
            var movvy = Engine.PhysicsTicksPerSecond / (int)Constants.TILES_PER_SECOND;
            _posCount += 1;
            Position = _curPos.Lerp(
                _walkGoals[0],
                (float)_posCount / (float)movvy
            );
            GD.Print(_curPos, ": ", Position);
            if (_posCount > movvy)
            {
                _curPos = _walkGoals[0];
                _walkGoals.RemoveAt(0);
                _posCount = 0;
                GD.Print("pop");
            }
        }
    }

    public void MoveTo(List<Vector3I> goal)
    {
        if (goal.Count > 0)
        {
            _curPos = goal[0];
            goal.RemoveAt(0);
            _posCount = 0;
            _walkGoals = goal;
        }
    }
}
