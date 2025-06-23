using Godot;
using System;
using System.Runtime.InteropServices;

public partial class MainMenu : Node3D
{
    NetworkManager nw;

    public override void _Ready()
    {
        GD.Print("==================================");
        this.nw = GetNode<NetworkManager>("/root/NetworkManager");

        GetNode<Button>("CanvasLayer/Button").Pressed += () =>
        {
            nw.Inner.SendChatMessage("hello everybody!");
        };

        GetNode<Button>("CanvasLayer/MainMenu/Host/Button").Pressed += () =>
        {
            if (NetworkClient.StartHostLoop(4000))
                nw.Connect("localhost:4000");

        };

        GetNode<Button>("CanvasLayer/MainMenu/Connect/Button").Pressed += () =>
        {
            nw.Connect("localhost:4000");
        };
    }

    public override void _Process(double delta)
    {
    }
}
