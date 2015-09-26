#version 330 core

precision highp float;

in  vec3 ex_color;
in  vec3 ex_norm;
out vec4 gl_FragColor;

void main(void) {
	gl_FragColor = vec4(ex_color, 1.0);
}
