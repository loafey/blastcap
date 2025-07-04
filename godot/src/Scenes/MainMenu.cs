using System;
using System.Collections.Generic;
using Godot;

public partial class MainMenu : Node3D {
    private NetworkManager nw;

    private HashSet<String> players = new HashSet<string>();

    [Export]
    public ChatBox ChatBox;

    [Export]
    public VBoxContainer PlayerList;

    [Export]
    public VBoxContainer MapList;

    [Export]
    public Button HostButton;

    [Export]
    public Button ConnectButton;

    private void drawPlayerList() {
        foreach (var child in PlayerList.GetChildren()) child.QueueFree();
        foreach (var player in players) {
            var lab = new Label();
            lab.Text = player;
            PlayerList.AddChild(lab);
        }
    }

    private void onConnect() {
        nw.Inner.OnChatMessage += (user, msg) => ChatBox.ShowMessage($"{user}: {msg}");
        nw.Inner.OnNewUser += (user) => {
            ChatBox.ShowMessage($"{user} joined");
            players.Add(user);
            drawPlayerList();
        };
        nw.Inner.OnUserLeft += (user) => {
            ChatBox.ShowMessage($"{user} left");
            players.Remove(user);
            drawPlayerList();
        };
        nw.Inner.OnStatus += (count, diff) => { };
        nw.Inner.OnPlayerList += (playerList) => {
            players.Clear();
            foreach (var player in playerList) players.Add(player);
            drawPlayerList();
        };
        nw.Inner.OnNotifyHost += () => nw.Inner.SendRequestMapList();
        nw.Inner.OnMapList += (list) => {
            MapList.Visible = true;
            foreach (var map in list) {
                var button = new Button();
                button.Text = map;
                button.Pressed += () => {
                    nw.Inner.SendStartMap(map);
                };
                MapList.AddChild(button);
            }
        };
        nw.Inner.OnStartMap += (map) => {
            GD.Print($"Starting map: {map}");
            GetTree().ChangeSceneToFile("res://scenes/Game.tscn");
        };
    }

    public override void _Ready() {
        GD.Print("==================================");
        nw = GetNode<NetworkManager>("/root/NetworkManager");

        HostButton.Pressed += () => {
            if (NetworkClient.StartHostLoop(4000)) {
                nw.Connect("localhost:4000");
                onConnect();
            }
        };

        ConnectButton.Pressed += () => {
            nw.Connect("localhost:4000");
            onConnect();
        };
    }


    public override void _Process(double delta) {
    }
}
