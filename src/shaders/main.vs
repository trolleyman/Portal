#version 330 core

uniform mat4 mvp;
uniform sampler2D textureID;

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec3 in_norm;

out vec3 ex_color;
out vec4 ex_norm;

void main() {
	gl_Position = mvp * vec4(in_pos, 1.0);
	
	vec4 color = texture(textureID, in_uv)
	ex_color = vec3(color.x, color.y, color.z);
	ex_norm = in_norm;
}
