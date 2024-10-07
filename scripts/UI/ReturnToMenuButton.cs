using Godot;
using System;

public partial class ReturnToMenuButton : BaseButton
{
	// Called when the node enters the scene tree for the first time.
	public override void _Ready()
	{
	}

	// Called every frame. 'delta' is the elapsed time since the previous frame.
	public override void _Process(double delta)
	{
		if (ButtonPressed)
		{
			GetTree().Paused = false;
			GetTree().ChangeSceneToFile("res://ui/mainMenu.tscn");
		}
	}
}
