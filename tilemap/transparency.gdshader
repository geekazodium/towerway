shader_type canvas_item;
render_mode blend_mix;
uniform float transparency;

void vertex() {
	// Called for every vertex the material is visible on.
}

void fragment() {
	COLOR = texture(TEXTURE, UV) * vec4(1, 1, 1, transparency);
}

//void light() {
	// Called for every pixel for every light affecting the material.
	// Uncomment to replace the default light processing function with this one.
//}
