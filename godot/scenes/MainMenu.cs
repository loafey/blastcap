using Godot;
using System;
using System.Runtime.InteropServices;

public partial class MainMenu : Node3D
{
    NetworkManager nw;

    private void showMessage(string msg)
    {
        var chatList = GetNode<VBoxContainer>("CanvasLayer/ChatBox/Messages/List");
        var label = new Label();
        label.Text = msg;
        chatList.AddChild(label);
    }

    public override void _Ready()
    {
        GD.Print("==================================");
        nw = GetNode<NetworkManager>("/root/NetworkManager");

        GetNode<Button>("CanvasLayer/MainMenu/Host/Button").Pressed += () =>
        {
            if (NetworkClient.StartHostLoop(4000))
                nw.Connect("localhost:4000");

        };

        GetNode<Button>("CanvasLayer/MainMenu/Connect/Button").Pressed += () =>
        {
            nw.Connect("localhost:4000");
        };

        GetNode<LineEdit>("CanvasLayer/ChatBox/Edit/Edit").TextSubmitted += (text) =>
        {
            nw.Inner.SendChatMessage(text);
            GetNode<LineEdit>("CanvasLayer/ChatBox/Edit/Edit").Text = "";
        };

        nw.ChatMessage += (user, msg) => showMessage($"{user}: {msg}");
        nw.UserJoin += (user) => showMessage($"{user} joined");
        nw.UserLeave += (user) => showMessage($"{user} left");


    }

    public override void _Process(double delta)
    {
    }
}
