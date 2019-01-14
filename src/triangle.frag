#version 430 core

out vec4 Color;

layout(location = 2) uniform vec2 resolution;
layout(location = 3) uniform vec3 cam_pos;
layout(location = 4) uniform float time;

const int MAX_MARCHING_STEPS = 255;
const float MIN_DIST = 0.0;
const float MAX_DIST = 100.0;
const float EPSILON = 0.0001;

float sphereSDF(vec3 samplePoint) {
    return length(samplePoint) - 1.0;
}

float sceneSDF(vec3 samplePoint) {
    return sphereSDF(samplePoint);
}

float shortestDistanceToSurface(vec3 eye, vec3 marchingDirection, float start, float end) {
    float depth = start;
    for (int i = 0; i < MAX_MARCHING_STEPS; i++) {
        float dist = sceneSDF(eye + depth * marchingDirection);
        if (dist < EPSILON) {
			return depth;
        }
        depth += dist;
        if (depth >= end) {
            return end;
        }
    }
    return end;
}

vec3 rayDirection(float fieldOfView, vec2 size, vec2 fragCoord) {
    vec2 xy = fragCoord - size / 2.0;
    float z = size.y / tan(radians(fieldOfView) / 2.0);
    return normalize(vec3(xy, -z));
}


void main()
{
	vec3 dir = rayDirection(45.0, resolution/*vec2(900, 700)*/, gl_FragCoord.xy);
    vec3 eye = vec3(0.0, 0.0, 5.0);
    float dist = shortestDistanceToSurface(eye, dir, MIN_DIST, MAX_DIST);

    if (dist > MAX_DIST - EPSILON) {
        // Didn't hit anything
        Color = vec4(0.0, 0.0, 0.0, 0.0);
		return;
    }

    Color = vec4(1.0, 0.0, 0.0, 1.0);
}