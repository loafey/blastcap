using Godot;
using System;
using System.Runtime.InteropServices;

public partial class MainMenu : Node3D
{
    private NetworkClient _nc = null;

    public override void _Ready()
    {
        GD.Print("==================================");
        NetworkClient.StartHostLoop(4000);
        this._nc = NetworkClient.StartClientLoop("localhost:4000");

        GetNode<Button>("CanvasLayer/Button").Pressed += () =>
        {
            this._nc.SendChatMessage("yoo");
            this._nc.SendPing();
        };
    }

    public override void _Process(double delta)
    {
        base._Process(delta);
        this._nc.Poll();
    }
}
