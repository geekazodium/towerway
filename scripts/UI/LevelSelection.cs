using Godot;
using System;

public partial class LevelSelection : TextureRect
{
	// Called when the node enters the scene tree for the first time.
	bool inputSwitch;
	float currentAlpha;
	public override void _Ready()
	{
		Visible = false;
		inputSwitch = false;
		currentAlpha = 0f;
	}

	// Called every frame. 'delta' is the elapsed time since the previous frame.
	public override void _Process(double delta)
	{
		if(inputSwitch)
		{
			Modulate = new Color(1, 1, 1, currentAlpha);
			currentAlpha +=(float)delta*2;
			if (currentAlpha > 1)
			{
				currentAlpha = 1;
				inputSwitch = false;
			}
		}
	}
	public override void _Input(InputEvent @event)
	{
		if(@event is InputEventKey inputEventKey)
		{
			if (!Visible && inputEventKey.Pressed)
			{
				Visible = true;
				Modulate = new Color(1, 1, 1, currentAlpha);
				inputSwitch = true;
			}
		}
		if(@event is InputEventMouseButton inputEventMouseButton)
		{
			if (!Visible && inputEventMouseButton.Pressed)
			{
				Visible = true;
				Modulate = new Color(1, 1, 1, currentAlpha);
				inputSwitch = true;
			}
		}
	}
}
