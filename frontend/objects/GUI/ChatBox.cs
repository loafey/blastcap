using Godot;

public partial class ChatBox : VBoxContainer {
    private NetworkManager nw;
    [Export]
    public LineEdit LineEdit;
    [Export]
    public VBoxContainer ChatList;
    public bool IsFocused => this.LineEdit.HasFocus();

    public void ShowMessage(string msg) {
        var label = new Label {
            Text = msg
        };
        this.ChatList.AddChild(label);
        this.ChatList.MoveChild(label, 0);
    }

    public override void _Ready() {
        base._Ready();
        this.nw = this.GetNode<NetworkManager>("/root/NetworkManager");
        this.LineEdit.TextSubmitted += (text) => {
            this.nw.Inner.SendChatMessage(text);
            this.LineEdit.Text = "";
        };
    }

}
