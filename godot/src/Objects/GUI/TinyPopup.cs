using Godot;
using System;

public partial class TinyPopup : Control {
    [Export]
    public string Text = "Popup";
    [Export]
    public Label Label;
    [Export]
    public AnimationPlayer AnimationPlayer;

    public override void _Ready() {
        Label.Text = Text;
        AnimationPlayer.Play("Popup");
        AnimationPlayer.AnimationFinished += (anim) => {
            if (anim == "Popup") QueueFree();
        };
    }
}
