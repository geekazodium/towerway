using Godot;
using System;

public partial class Autoplay : AudioStreamPlayer
{
    public override void _Ready(){
        Play();
    }
}
