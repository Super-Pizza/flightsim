#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 outColor;

void main() {
    gl_Position = vec4(position * vec3(1.0,-1.0, 1.0), 1.0);
    outColor = inColor;
}
