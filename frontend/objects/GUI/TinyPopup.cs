using Godot;

public partial class TinyPopup : Control {
    [Export]
    public string Text = "Popup";
    [Export]
    public Label Label;
    [Export]
    public AnimationPlayer AnimationPlayer;

    public override void _Ready() {
        this.Label.Text = this.Text;
        this.AnimationPlayer.Play("Popup");
        this.AnimationPlayer.AnimationFinished += (anim) => {
            if (anim == "Popup") {
                this.QueueFree();
            }
        };
    }
}
