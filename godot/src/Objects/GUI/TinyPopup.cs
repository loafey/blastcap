using Godot;
using System;

public partial class TinyPopup : Control
{
    [Export]
    public string Text = "Popup";

    public override void _Ready()
    {
        var node = GetNode<Label>("Panel/Label");
        node.Text = Text;
        var player = GetNode<AnimationPlayer>("AnimationPlayer");
        player.Play("Popup");
        player.AnimationFinished += (anim) =>
        {
            if (anim == "Popup") QueueFree();
        };
    }
}
