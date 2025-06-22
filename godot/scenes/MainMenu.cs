using Godot;
using System;
using System.Runtime.InteropServices;

public partial class MainMenu : Node3D
{
    public override void _Ready()
    {
        GD.Print("==================================");
        FFI.StartHostLoop(4000);
        var client = FFI.StartClientLoop("localhost:4000");
        client.SendChatMessage();
    }
}
