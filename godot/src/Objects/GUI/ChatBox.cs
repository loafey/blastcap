using Godot;
using System;

public partial class ChatBox : VBoxContainer {
    private NetworkManager nw;
    [Export]
    public LineEdit LineEdit;
    [Export]
    public VBoxContainer ChatList;
    public bool IsFocused { get => LineEdit.HasFocus(); }

    public void ShowMessage(string msg) {
        var label = new Label();
        label.Text = msg;
        ChatList.AddChild(label);
        ChatList.MoveChild(label, 0);
    }

    public override void _Ready() {
        base._Ready();
        nw = GetNode<NetworkManager>("/root/NetworkManager");
        LineEdit.TextSubmitted += (text) => {
            nw.Inner.SendChatMessage(text);
            LineEdit.Text = "";
        };
    }

}
