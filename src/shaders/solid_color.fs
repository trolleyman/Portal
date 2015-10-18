#version 330 core

precision highp float;

out vec4 gl_FragColor;

in vec4 ex_color;

void main(void) {
	gl_FragColor = ex_color;
}
