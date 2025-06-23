using Godot;
using System;

public partial class NetworkManager : Node
{
    private NetworkClient _inner;
    public NetworkClient Inner
    {
        get => _inner;
    }

    public void Connect(String addr)
    {
        if (this.Inner != null) return;
        this._inner = NetworkClient.StartClientLoop(
            addr,
            onFail: (err) =>
            {
                GD.PrintErr(err);
                this._inner.Drop();
                this._inner = null;
            },
            PongFn: () => GD.Print("PONG! :)"),
            ChatMessageFn: (user, msg) => GD.Print($"{user}: {msg}"),
            NewUserFn: (user) => GD.Print($"{user} connected"),
            UserLeftFn: (user) => GD.Print($"{user} left"),
            StatusFn: (userCount, tickRate) => {/*GD.Print($"STATUS: {userCount}U/{tickRate}S")*/}
        );
    }

    public override void _Process(double delta)
    {
        base._Process(delta);
        if (_inner != null) _inner.Poll();
    }
}
