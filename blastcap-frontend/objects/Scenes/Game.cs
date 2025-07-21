using Godot;
using System;
using System.Collections.Generic;
using System.Diagnostics;


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
    private readonly Dictionary<int, Node3D> _layers = [];
    [Export]
    public PackedScene ExplosionScene;
    [Export]
    public PackedScene SoundEffect;
    [Export]
    public Node3D Temporaries;
    private bool _myTurn;
    private string _currentAbility = null;
    private readonly Random _random = new();
    private Stopwatch _rtt = new();
    private ulong _tickCount = 0;



    private void SpawnCube(Node3D parent, Vector3 pos) {
        var floor = new MeshInstance3D();
        var mesh = new BoxMesh {
            // CenterOffset = new Vector3(0.5f, 0, 0.5f),
            Size = new Vector3(1f, 1f, 1f)
        };
        floor.Mesh = mesh;
        var mat = new StandardMaterial3D();
        var color = this._random.NextInt64() % 256 / 256.0f;
        mat.AlbedoColor = new Color(color, color, color);
        floor.MaterialOverride = mat;
        var position = new Vector3(pos.X, pos.Y, pos.Z);
        floor.Position = position + new Vector3(0.5f, 0.5f, 0.5f);

        var coll = new StaticBody3D {
            // Position = new Vector3(0.5f, -0.5f, 0.5f)
        };
        var collShape = new CollisionShape3D();
        var shape = new BoxShape3D();
        collShape.Shape = shape;

        coll.AddChild(collShape);
        floor.AddChild(coll);
        parent.AddChild(floor);
    }

    public override void _Ready() {
        base._Ready();
        this.nw = this.GetNode<NetworkManager>("/root/NetworkManager");
        this.nw.Inner.SendNotifyReady();

        this.nw.Inner.OnChatMessage += (user, msg) => this.ChatBox.ShowMessage($"{user}: {msg}");
        this.nw.Inner.OnNewUser += (user) => {
            this.ChatBox.ShowMessage($"{user} joined");
        };
        this.nw.Inner.OnUserLeft += (user) => {
            this.ChatBox.ShowMessage($"{user} left");
        };

        this.nw.Inner.OnSpawnActor += (mine, name, id, x, y, z, abilities, movement, health, maxHealth) => {
            var node = this.ActorScene.Instantiate<Actor>();
            node.Position = new Vector3(x, y, z);
            node.ActorName = name;
            node.Name = id.ToString();
            node.Abilities = abilities;
            node.MaxHealth = maxHealth;
            node.Health = health;
            node.BaseMovement = movement;
            this.ActorHolder.AddChild(node);
            if (mine) {
                this.PC.MyActor = node;
                foreach (var item in abilities) {
                    var tt = Data.Abilities[item];
                    this.PC.AddAbilityButton(
                        item,
                        tt,
                        () => {
                            this._currentAbility = item;
                            this.PC.CurrentAbility = item;
                        }
                    );
                }
            }
        };

        this.nw.Inner.OnYourTurn += (id, movement) => {
            // _chatBox.ShowMessage("YOUR TURN");
            this.PC.DisplayTinyPopup("YOUR TURN");
            this.PC.MyTurn = true;
            this._myTurn = true;
            this.PC.Movement = (movement, this.PC.MyActor.BaseMovement);
        };
        this.nw.Inner.OnActorTurn += (id) => {
            var actor = this.ActorHolder.GetNode<Actor>(id.ToString()).ActorName;
            this.PC.DisplayTinyPopup($"{actor.ToUpperInvariant()}'S TURN");
            // _chatBox.ShowMessage($"{actor.ToUpperInvariant()}'S TURN");
            this.PC.MyTurn = false;
            this._myTurn = false;
        };

        this.nw.Inner.OnMoveActor += (id, movement, xList, yList, zList) => {
            var actor = this.ActorHolder.GetNode<Actor>(id.ToString());
            var goals = new List<Vector3I>();
            for (var i = 0; i < xList.Count; i++) {
                var x = xList[i];
                var y = yList[i];
                var z = zList[i];
                goals.Add(new Vector3I((int)x, (int)y, (int)z));
            }
            actor.MoveTo(goals);
            if (this._myTurn) {
                this.PC.Movement = (movement, this.PC.MyActor.BaseMovement);
            }
        };

        this.nw.Inner.OnAction += (action, actorIndex, targetIndex, targetDamage, time) => {
            var children = this.ActorHolder.GetChildren();
            var actor = (Actor)children[(int)actorIndex];
            var target = (Actor)children[(int)targetIndex];
            var middle = (actor.Position + target.Position) / 2;

            var node = this.ExplosionScene.Instantiate<Explosion>();
            node.Position = middle;
            this.Temporaries.AddChild(node);

            var sound = this.SoundEffect.Instantiate<Node3D>();
            sound.Position = middle;
            this.Temporaries.AddChild(sound);
            target.Health -= targetDamage;
            target.Visible = target.Health > 0;
        };

        this.PC.EndTurnPressed = () => {
            this.nw.Inner.SendEndTurn();
            this._currentAbility = null;
            this.PC.CurrentAbility = null;
        };

        this.nw.Inner.OnSpawnMap += (xList, yList, zList) => {
            for (var i = 0; i < xList.Count; i++) {
                var key = (int)yList[i];
                if (!this._layers.ContainsKey(key)) {
                    var par = new Node3D();
                    this._layers[key] = par;
                    this.WorldMeshHolder.AddChild(par);
                }
                var parent = this._layers[key];
                var pos = new Vector3(xList[i], yList[i], zList[i]);
                this.SpawnCube(parent, pos);
            }
        };

        this.nw.Inner.SendPing();
        this._rtt = new Stopwatch();
        this._tickCount = 0;
        this.nw.Inner.OnPong += () => {
            this.PC.RTT = (this._rtt.Elapsed.Milliseconds, this._tickCount);
            var timer = this.GetTree().CreateTimer(1);
            timer.Timeout += () => {
                this._rtt = new Stopwatch();
                this._tickCount = 0;
                this._rtt.Start();
                this.nw.Inner.SendPing();
            };
        };
    }

    public override void _UnhandledInput(InputEvent @event) {
        base._UnhandledInput(@event);
        if (Input.IsActionJustPressed("actor_walk")) {
            var mp = this.GetViewport().GetMousePosition();
            var space = this.GetWorld3D().DirectSpaceState;
            var cam = this.PC.Camera;

            var origin = cam.ProjectRayOrigin(mp);
            var end = origin + (cam.ProjectRayNormal(mp) * 10000);
            var query = PhysicsRayQueryParameters3D.Create(origin, end);
            query.CollideWithAreas = true;
            query.CollisionMask = 0b00000000_00000000_00000000_00000001;

            var result = space.IntersectRay(query);
            if (result.Count == 0) {
                return;
            }

            var pos = (Vector3)result["position"];
            // pos.Y += 1;
            // GD.Print($"{(nuint)pos.X}, {(nuint)pos.Y}, {(nuint)pos.Z}");
            if (this._myTurn && this._currentAbility != null) {
                this.nw.Inner.SendAction(this._currentAbility, (nuint)pos.X, (nuint)pos.Y, (nuint)pos.Z);
                this._currentAbility = null;
                this.PC.CurrentAbility = null;
            }
        }
    }


    public override void _Process(double delta) {
        base._Process(delta);
        this._tickCount += 1;
    }
}
