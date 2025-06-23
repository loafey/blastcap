using Godot;
using System;
using System.Runtime.InteropServices;

public partial class MainMenu : Node3D
{
    private NetworkClient _nc = null;

    private void Connect(String addr)
    {
        if (this._nc != null) return;
        this._nc = NetworkClient.StartClientLoop(
            addr,
            PongFn: () => GD.Print("PONG! :)"),
            ChatMessageFn: (user, msg) => GD.Print($"{user}: {msg}"),
            NewUserFn: (user) => GD.Print($"{user} connected"),
            UserLeftFn: (user) => GD.Print($"{user} left"),
            StatusFn: (userCount, tickRate) => {/*GD.Print($"STATUS: {userCount}U/{tickRate}S")*/}
        );
    }

    public override void _Ready()
    {
        GD.Print("==================================");

        GetNode<Button>("CanvasLayer/Button").Pressed += () =>
        {
            this._nc.SendChatMessage("hello everybody!");
        };

        GetNode<Button>("CanvasLayer/MainMenu/Host/Button").Pressed += () =>
        {
            if (NetworkClient.StartHostLoop(4000)) Connect("localhost:4000");

        };

        GetNode<Button>("CanvasLayer/MainMenu/Connect/Button").Pressed += () =>
        {
            Connect("localhost:4000");
        };
    }

    public override void _Process(double delta)
    {
        base._Process(delta);
        if (this._nc != null) this._nc.Poll();
    }
}
