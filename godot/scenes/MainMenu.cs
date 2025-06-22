using Godot;
using System;
using System.Runtime.InteropServices;

public partial class MainMenu : Node3D
{
    public override void _Ready()
    {
        GD.Print("==================================");
        NetworkClient.StartHostLoop(4000);
        var client = NetworkClient.StartClientLoop("localhost:4000");

        GetNode<Button>("CanvasLayer/Button").Pressed += () =>
        {
            client.SendChatMessage("yoo");
            client.SendPing();
        };
    }
}
