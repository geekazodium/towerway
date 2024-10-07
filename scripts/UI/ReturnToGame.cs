using Godot;
using System;

public partial class ReturnToGame : Button
{
    [Export] Control root_node;
	public override void _Process(double delta)
	{
		if (ButtonPressed)
		{
            root_node.Hide();
            root_node.QueueFree();
		}
	}
}