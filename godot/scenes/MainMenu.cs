using System;
using Godot;

public partial class MainMenu : Node3D
{
    private NetworkManager nw;

    private void showMessage(string msg)
    {
        var chatList = GetNode<VBoxContainer>("CanvasLayer/ChatBox/Messages/List");
        var label = new Label();
        label.Text = msg;
        chatList.AddChild(label);
    }

    private void onConnect()
    {
        nw.Inner.OnChatMessage += (user, msg) => showMessage($"{user}: {msg}");
        nw.Inner.OnNewUser += (user) => showMessage($"{user} joined");
        nw.Inner.OnUserLeft += (user) => showMessage($"{user} left");
        nw.Inner.OnStatus += (count, diff) => { };
        nw.Inner.OnPlayerList += (players) =>
        {
            GD.Print("Player list:");
            foreach (var player in players) GD.Print($"\t{player}");
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
