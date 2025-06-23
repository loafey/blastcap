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

    public delegate void OnChatMessage(String user, String msg);
    public event OnChatMessage ChatMessage;

    public delegate void OnUserJoin(String user);
    public event OnUserJoin UserJoin;

    public delegate void OnUserLeave(String user);
    public event OnUserLeave UserLeave;

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
            ChatMessageFn: (user, msg) => ChatMessage(user, msg),
            NewUserFn: (user) => UserJoin(user),
            UserLeftFn: (user) => UserLeave(user),
            StatusFn: (userCount, tickRate) => {/*GD.Print($"STATUS: {userCount}U/{tickRate}S")*/}
        );
    }

    public override void _Process(double delta)
    {
        base._Process(delta);
        if (_inner != null) _inner.Poll();
    }
}
