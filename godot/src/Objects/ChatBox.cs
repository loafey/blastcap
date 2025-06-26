using Godot;
using System;

public partial class ChatBox : VBoxContainer
{
    private NetworkManager nw;
    private LineEdit _le;
    private VBoxContainer _cl;
    public bool IsFocused { get => _le.HasFocus(); }

    public void ShowMessage(string msg)
    {
        var label = new Label();
        label.Text = msg;
        _cl.AddChild(label);
        _cl.MoveChild(label, 0);
    }

    public override void _Ready()
    {
        base._Ready();
        nw = GetNode<NetworkManager>("/root/NetworkManager");
        _le = GetNode<LineEdit>("Edit/Edit");
        _cl = GetNode<VBoxContainer>("Messages/List");
        _le.TextSubmitted += (text) =>
        {
            nw.Inner.SendChatMessage(text);
            _le.Text = "";
        };
    }

}
