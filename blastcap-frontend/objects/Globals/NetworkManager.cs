using Godot;
using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;

public partial class NetworkManager : Node {
    public NetworkClient Inner { get; private set; }

    public bool IsHost => NetworkClient.IsHost;

    public override void _Ready() {
        this.Inner = new NetworkClient((err) => {
            GD.PrintErr(err);
            this.Inner.Drop();
            this.Inner = null;
        });
    }

    public void Connect(string addr) {
        if (!this.Inner.IsConnected()) {
            this.Inner.Connect(addr);
        }
    }

    public override void _Process(double delta) {
        base._Process(delta);
        this.Inner?.Poll();
    }
}
