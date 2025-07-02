using Godot;
using System;
using System.Collections.Generic;

public partial class Game : Node3D
{
    private NetworkManager nw;
    private PackedScene _actorScene;
    private Node3D _actorHolder;
    private ChatBox _chatBox;
    private PlayerCamera _playerCamera;
    private bool _myTurn;

    private void setupDebugScene()
    {
        var parent = GetNode<Node3D>("Debug");
        var rand = new Random();
        for (int x = 0; x < 16; x++)
        {
            for (int y = 0; y < 16; y++)
            {
                var floor = new MeshInstance3D();
                var mesh = new PlaneMesh();
                mesh.CenterOffset = new Vector3(0.5f, 0, 0.5f);
                mesh.Size = new Vector2(1f, 1f);
                floor.Mesh = mesh;
                var mat = new StandardMaterial3D();
                var color = (rand.NextInt64() % 256) / 256.0f;
                mat.AlbedoColor = new Color(color, color, color);
                floor.MaterialOverride = mat;
                var position = new Vector3(x, 0, y);
                floor.Position = position;

                var coll = new StaticBody3D();
                coll.Position = new Vector3(0.5f, 0, 0.5f);
                var collShape = new CollisionShape3D();
                var shape = new BoxShape3D();
                shape.Size = new Vector3(1, 0.2f, 1);
                collShape.Shape = shape;

                coll.AddChild(collShape);
                floor.AddChild(coll);
                parent.AddChild(floor);
            }
        }
    }

    public override void _Ready()
    {
        base._Ready();
        _actorScene = GD.Load<PackedScene>("uid://dmad3dtbb46yk");
        _actorHolder = GetNode<Node3D>("Actors");
        _chatBox = GetNode<ChatBox>("CanvasLayer/ChatBox");
        _playerCamera = GetNode<PlayerCamera>("PlayerCamera");
        nw = GetNode<NetworkManager>("/root/NetworkManager");
        nw.Inner.SendNotifyReady();

        nw.Inner.OnChatMessage += (user, msg) => _chatBox.ShowMessage($"{user}: {msg}");
        nw.Inner.OnNewUser += (user) =>
        {
            _chatBox.ShowMessage($"{user} joined");
        };
        nw.Inner.OnUserLeft += (user) =>
        {
            _chatBox.ShowMessage($"{user} left");
        };

        nw.Inner.OnSpawnPlayer += (name, id, x, y) =>
        {
            var node = _actorScene.Instantiate<Actor>();
            _actorHolder.AddChild(node);
            node.Position = new Vector3(x, 0, y);
            node.ActorName = name;
            node.Name = id.ToString();
        };

        nw.Inner.OnYourTurn += (id) =>
        {
            // _chatBox.ShowMessage("YOUR TURN");
            _playerCamera.DisplayTinyPopup("YOUR TURN");
            _playerCamera.MyTurn = true;
            _myTurn = true;
        };
        nw.Inner.OnActorTurn += (id) =>
        {
            var actor = _actorHolder.GetNode<Actor>(id.ToString()).ActorName;
            _playerCamera.DisplayTinyPopup($"{actor.ToUpperInvariant()}'S TURN");
            // _chatBox.ShowMessage($"{actor.ToUpperInvariant()}'S TURN");
            _playerCamera.MyTurn = false;
            _myTurn = false;
        };

        nw.Inner.OnMoveActor += (id, xList, yList) =>
        {
            var actor = _actorHolder.GetNode<Actor>(id.ToString());
            var goals = new List<Vector3I>();
            for (int i = 0; i < xList.Count; i++)
            {
                var x = xList[i];
                var y = yList[i];
                goals.Add(new Vector3I((int)x, 0, (int)y));
            }
            actor.MoveTo(goals);
        };

        setupDebugScene();
    }

    public override void _UnhandledInput(InputEvent @event)
    {
        base._UnhandledInput(@event);
        if (_myTurn && Input.IsActionJustPressed("actor_walk"))
        {
            var mp = GetViewport().GetMousePosition();
            var space = GetWorld3D().DirectSpaceState;
            var cam = _playerCamera.Camera;

            var origin = cam.ProjectRayOrigin(mp);
            var end = origin + cam.ProjectRayNormal(mp) * 10000;
            var query = PhysicsRayQueryParameters3D.Create(origin, end);
            query.CollideWithAreas = true;
            query.CollisionMask = 0b00000000_00000000_00000000_00000001;

            var result = space.IntersectRay(query);
            if (result.Count == 0) return;

            Vector3 pos = (Vector3)result["position"];
            nw.Inner.SendMoveActor((nuint)pos.X, (nuint)pos.Z);
        }
    }


    public override void _Process(double delta)
    {
        base._Process(delta);

    }
}
