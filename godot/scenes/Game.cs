using Godot;
using System;
using System.Collections.Generic;


public partial class Game : Node3D {
    private NetworkManager nw;
    [Export]
    public PackedScene ActorScene;
    [Export]
    public Node3D ActorHolder;
    [Export]
    public ChatBox ChatBox;
    [Export]
    public PlayerCamera PC;
    [Export]
    public Node3D WorldMeshHolder;
    private bool _myTurn;
    private string _currentAbility = null;

    private void setupDebugScene() {
        var rand = new Random();
        for (int x = 0; x < 16; x++) {
            for (int y = 0; y < 16; y++) {
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
                WorldMeshHolder.AddChild(floor);
            }
        }
    }

    public override void _Ready() {
        base._Ready();
        nw = GetNode<NetworkManager>("/root/NetworkManager");
        nw.Inner.SendNotifyReady();

        nw.Inner.OnChatMessage += (user, msg) => ChatBox.ShowMessage($"{user}: {msg}");
        nw.Inner.OnNewUser += (user) => {
            ChatBox.ShowMessage($"{user} joined");
        };
        nw.Inner.OnUserLeft += (user) => {
            ChatBox.ShowMessage($"{user} left");
        };

        nw.Inner.OnSpawnActor += (mine, name, id, x, y, abilities) => {
            var node = ActorScene.Instantiate<Actor>();
            ActorHolder.AddChild(node);
            node.Position = new Vector3(x, 0, y);
            node.ActorName = name;
            node.Name = id.ToString();
            node.Abilities = abilities;
            if (mine) {
                PC.MyActor = node;
                foreach (var item in abilities) {
                    var tt = Data.Abilities[item];
                    PC.AddAbilityButton(
                        item,
                        tt,
                        () => {
                            _currentAbility = item;
                            PC.CurrentAbility = item;
                        }
                    );
                }
            }
        };

        nw.Inner.OnYourTurn += (id) => {
            // _chatBox.ShowMessage("YOUR TURN");
            PC.DisplayTinyPopup("YOUR TURN");
            PC.MyTurn = true;
            _myTurn = true;
        };
        nw.Inner.OnActorTurn += (id) => {
            var actor = ActorHolder.GetNode<Actor>(id.ToString()).ActorName;
            PC.DisplayTinyPopup($"{actor.ToUpperInvariant()}'S TURN");
            // _chatBox.ShowMessage($"{actor.ToUpperInvariant()}'S TURN");
            PC.MyTurn = false;
            _myTurn = false;
        };

        nw.Inner.OnMoveActor += (id, xList, yList) => {
            var actor = ActorHolder.GetNode<Actor>(id.ToString());
            var goals = new List<Vector3I>();
            for (int i = 0; i < xList.Count; i++) {
                var x = xList[i];
                var y = yList[i];
                goals.Add(new Vector3I((int)x, 0, (int)y));
            }
            actor.MoveTo(goals);
        };

        nw.Inner.OnAbilityMap += (map) => { Data.Abilities = map; };

        PC.EndTurnPressed = () => {
            nw.Inner.SendEndTurn(); _currentAbility = null;
            PC.CurrentAbility = null;
        };

        setupDebugScene();
    }

    public override void _UnhandledInput(InputEvent @event) {
        base._UnhandledInput(@event);
        if (_myTurn && Input.IsActionJustPressed("actor_walk")) {
            var mp = GetViewport().GetMousePosition();
            var space = GetWorld3D().DirectSpaceState;
            var cam = PC.Camera;

            var origin = cam.ProjectRayOrigin(mp);
            var end = origin + cam.ProjectRayNormal(mp) * 10000;
            var query = PhysicsRayQueryParameters3D.Create(origin, end);
            query.CollideWithAreas = true;
            query.CollisionMask = 0b00000000_00000000_00000000_00000001;

            var result = space.IntersectRay(query);
            if (result.Count == 0) return;

            Vector3 pos = (Vector3)result["position"];
            if (_currentAbility != null) {
                nw.Inner.SendAction(_currentAbility, (nuint)pos.X, (nuint)pos.Z);
                _currentAbility = null;
                PC.CurrentAbility = null;
            }
        }
    }


    public override void _Process(double delta) {
        base._Process(delta);

    }
}
