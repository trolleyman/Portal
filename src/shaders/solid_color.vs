#version 330 core

uniform mat4 in_mvp;
uniform vec4 in_color;

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec3 attrib_color;

out vec4 ex_color;

void main() {
	gl_Position = in_mvp * vec4(in_pos, 1.0);
	ex_color = in_color;
}
