#version 150

uniform mat4 in_mvp;
in vec3 in_pos;
in vec3 in_color;

out vec3 ex_color;

void main() {
	gl_Position = in_mvp * vec4(in_pos, 1.0);
	ex_color = in_color;
}
