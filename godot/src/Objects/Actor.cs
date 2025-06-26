using Godot;
using System;

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
}
