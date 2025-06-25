using Godot;
using System;
using System.Collections.Generic;

public partial class NetworkManager : Node
{
    private NetworkClient _inner;
    public NetworkClient Inner
    {
        get => _inner;
    }

    public bool IsHost { get => NetworkClient.IsHost; }

    public void Connect(String addr)
    {
        if (this.Inner != null) return;
        this._inner = new NetworkClient(
            addr,
            onFail: (err) =>
            {
                GD.PrintErr(err);
                this._inner.Drop();
                this._inner = null;
            }
        );
    }

    public override void _Process(double delta)
    {
        base._Process(delta);
        if (_inner != null) _inner.Poll();
    }
}
