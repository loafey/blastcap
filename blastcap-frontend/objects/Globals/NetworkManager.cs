using Godot;
using System;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;

public partial class NetworkManager : Node {
    public NetworkClient Inner { get; private set; }

    public bool IsHost => NetworkClient.IsHost;

    public void Connect(string addr) {
        if (this.Inner != null) {
            return;
        }

        this.Inner = new NetworkClient(
            addr,
            onFail: (err) => {
                GD.PrintErr(err);
                this.Inner.Drop();
                this.Inner = null;
            }
        );
    }

    public override void _Process(double delta) {
        base._Process(delta);
        this.Inner?.Poll();
    }
}
