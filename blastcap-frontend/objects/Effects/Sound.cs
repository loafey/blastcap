using Godot;
using System;

public partial class Sound : AudioStreamPlayer3D {
    public override void _Ready() {
        base._Ready();
        this.Play();
        this.Finished += this.QueueFree;
    }

}
