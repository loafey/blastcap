using System;
using System.Collections.Generic;
using Godot;

public partial class MainMenu : Node3D {
    private NetworkManager nw;

    private HashSet<String> players = new HashSet<string>();

    private ChatBox _chatBox;

    private void drawPlayerList() {
        var playerList = GetNode<VBoxContainer>("CanvasLayer/PlayerList/Scroll/List");
        foreach (var child in playerList.GetChildren()) child.QueueFree();
        foreach (var player in players) {
            var lab = new Label();
            lab.Text = player;
            playerList.AddChild(lab);
        }
    }

    private void onConnect() {
        nw.Inner.OnChatMessage += (user, msg) => _chatBox.ShowMessage($"{user}: {msg}");
        nw.Inner.OnNewUser += (user) => {
            _chatBox.ShowMessage($"{user} joined");
            players.Add(user);
            drawPlayerList();
        };
        nw.Inner.OnUserLeft += (user) => {
            _chatBox.ShowMessage($"{user} left");
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
            var guiList = GetNode<VBoxContainer>("CanvasLayer/MainMenu/MapList");
            guiList.Visible = true;
            foreach (var map in list) {
                var button = new Button();
                button.Text = map;
                button.Pressed += () => {
                    nw.Inner.SendStartMap(map);
                };
                guiList.AddChild(button);
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
        _chatBox = GetNode<ChatBox>("CanvasLayer/ChatBox");

        GetNode<Button>("CanvasLayer/MainMenu/Host/Button").Pressed += () => {
            if (NetworkClient.StartHostLoop(4000)) {
                nw.Connect("localhost:4000");
                onConnect();
            }
        };

        GetNode<Button>("CanvasLayer/MainMenu/Connect/Button").Pressed += () => {
            nw.Connect("localhost:4000");
            onConnect();
        };
    }


    public override void _Process(double delta) {
    }
}
