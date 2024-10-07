using Godot;
using System;

public partial class AutoplayWithPitchShift : AudioStreamPlayer
{
    [Export] float min;
    [Export] float range;
    public override void _Ready(){
        PitchScale = min+Random.Shared.NextSingle() * range;
        Play();
    }
}