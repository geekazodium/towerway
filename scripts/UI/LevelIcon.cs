using Godot;
using System;

public partial class LevelIcon : TextureButton
{
	string sceneLocation;
	// Called when the node enters the scene tree for the first time.
	public override void _Ready()
	{
		GetMeta("Location",sceneLocation);
	}

	// Called every frame. 'delta' is the elapsed time since the previous frame.
	public override void _Process(double delta)
	{
		if (ButtonPressed)
		{
			GetTree().ChangeSceneToFile("res://level_1.tscn");
		}
	}
}
