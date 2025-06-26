using Godot;
using System;

public partial class Game : Node3D
{
    private NetworkManager nw;
    private PackedScene _actorScene;
    private Node3D _actorHolder;
    public override void _Ready()
    {
        base._Ready();
        _actorScene = GD.Load<PackedScene>("uid://dmad3dtbb46yk");
        _actorHolder = GetNode<Node3D>("Actors");
        nw = GetNode<NetworkManager>("/root/NetworkManager");
        nw.Inner.SendNotifyReady();

        nw.Inner.OnSpawnPlayer += (id, x, y) =>
        {
            var node = _actorScene.Instantiate<Node3D>();
            _actorHolder.AddChild(node);
            var pos = node.Position;
            pos.X = x;
            pos.Z = y;
            node.Position = pos;
        };
    }
}
