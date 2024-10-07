using Godot;
using System;

public partial class MainMenuSprite : Sprite2D
{
	float currentAlpha;
	int sign;
	// Called when the node enters the scene tree for the first time.
	public override void _Ready()
	{
		currentAlpha = 1;
		sign = -1;
	}

	// Called every frame. 'delta' is the elapsed time since the previous frame.
	public override void _Process(double delta)
	{
		Modulate = new Color(1,1,1,currentAlpha);
		currentAlpha=currentAlpha+sign*(float)delta/2;
		if(currentAlpha > 1)
		{
			sign = -1;
			currentAlpha = 1;
		}
		else if (currentAlpha < 0)
		{
			sign = 1;
			currentAlpha = 0;
		}
	}
}
