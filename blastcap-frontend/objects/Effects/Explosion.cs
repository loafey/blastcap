using Godot;
using System;

public partial class Explosion : Node3D {
    public override void _Ready() {
        this.GetNode<CpuParticles3D>("Particles").Emitting = true;
        this.GetNode<CpuParticles3D>("Particles").Finished += this.QueueFree;
    }
}
