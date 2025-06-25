using System;
using System.Collections.Generic;
using Godot;

public partial class MainMenu : Node3D
{
    private NetworkManager nw;

    private HashSet<String> players = new HashSet<string>();

    private void showMessage(string msg)
    {
        var chatList = GetNode<VBoxContainer>("CanvasLayer/ChatBox/Messages/List");
        var label = new Label();
        label.Text = msg;
        chatList.AddChild(label);
        chatList.MoveChild(label, 0);
    }

    private void drawPlayerList()
    {
        var playerList = GetNode<VBoxContainer>("CanvasLayer/PlayerList/List");
        foreach (var child in playerList.GetChildren()) child.QueueFree();
        foreach (var player in players)
        {
            var lab = new Label();
            lab.Text = player;
            playerList.AddChild(lab);
        }
    }

    private void onConnect()
    {
        nw.Inner.OnChatMessage += (user, msg) => showMessage($"{user}: {msg}");
        nw.Inner.OnNewUser += (user) =>
        {
            GD.Print("new player");
            showMessage($"{user} joined");
            players.Add(user);
            drawPlayerList();
        };
        nw.Inner.OnUserLeft += (user) =>
        {
            showMessage($"{user} left");
            players.Remove(user);
            drawPlayerList();
        };
        nw.Inner.OnStatus += (count, diff) => { };
        nw.Inner.OnPlayerList += (playerList) =>
        {
            GD.Print("set player list");
            players.Clear();
            foreach (var player in playerList) players.Add(player);
            drawPlayerList();
        };
    }

    public override void _Ready()
    {
        GD.Print("==================================");
        nw = GetNode<NetworkManager>("/root/NetworkManager");

        GetNode<Button>("CanvasLayer/MainMenu/Host/Button").Pressed += () =>
        {
            if (NetworkClient.StartHostLoop(4000))
            {
                nw.Connect("localhost:4000");
                onConnect();
            }
        };

        GetNode<Button>("CanvasLayer/MainMenu/Connect/Button").Pressed += () =>
        {
            nw.Connect("localhost:4000");
            onConnect();
        };

        GetNode<LineEdit>("CanvasLayer/ChatBox/Edit/Edit").TextSubmitted += (text) =>
        {
            nw.Inner.SendChatMessage(text);
            GetNode<LineEdit>("CanvasLayer/ChatBox/Edit/Edit").Text = "";
        };
    }

    public override void _Process(double delta)
    {
    }
}
