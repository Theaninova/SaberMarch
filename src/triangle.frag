#version 430 core

layout(location = 0) out vec3 Color;

layout(location = 2) uniform vec2 resolution;
layout(location = 3) uniform vec3 cam_pos;
layout(location = 4) uniform float time;

const int MAX_MARCHING_STEPS = 255;
const float MIN_DIST = 0.0;
const float MAX_DIST = 100.0;
const float EPSILON = 0.0001;

float dot2_3(vec3 v) {
    return dot(v,v);
}
float dot2_2(vec2 v) {
    return dot(v,v);
}

//SDFs
// http://www.iquilezles.org/www/articles/distfunctions/distfunctions.htm
// The MIT License
// Copyright © 2018 Inigo Quilez
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
float sphereSDF(vec3 p, float radius) {
    return length(p) - radius;
}
float boxSDF(vec3 p, vec3 s) {
    vec3 d = abs(p) - s;
    return length(max(d, 0.0)) + min(max(d.x, max(d.y, d.z)), 0.0);
}
float roundBoxSDF(vec3 p, vec3 s, float radius) {
    return boxSDF(p, s) - radius;
}
float torusSDF(vec3 p, vec2 s) {
    return length(vec2(length(p.xz) - s.x, p.y)) - s.y;
}
float cylinderSDF(vec3 p, vec3 s) {
    return length(p.xz - s.xy) - s.z;
}
float coneSDF(vec3 p, vec2 s) {
    return dot(s, vec2(length(p.xy), p.z));
}
float planeSDF(vec3 p, vec4 s_n) {
    return dot(p, s_n.xyz) + s_n.w;
}
float hexPrismSDF(vec3 p, vec2 s) {
    const vec3 k = vec3(-0.8660254, 0.5, 0.57735);
    p = abs(p);
    p.xy -= 2.0*min(dot(k.xy, p.xy), 0.0)*k.xy;
    vec2 d = vec2(length(p.xy - vec2(clamp(p.x, -k.z * s.x, k.z * s.x), s.x)) * sign(p.y - s.x), p.z - s.y);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}
float triPrismSDF(vec3 p, vec2 s) {
    vec3 q = abs(p);
    return max(q.z - s.y, max(q.x * 0.866025 + p.y * 0.5, -p.y) - s.x * 0.5);
}
float capsuleSDF(vec3 p, vec3 p_a, vec3 p_b, float r) {
    vec3 pa = p - p_a, ba = p_b - p_a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}
float verticalCapsuleSDF(vec3 p, float s, float r) {
    p.y -= clamp(p.y, 0.0, s);
    return length(p) - r;
}
float cylinderSDF(vec3 p, vec2 s) {
    vec2 d = abs(vec2(length(p.xz), p.y)) - s;
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}
float roundedCylinder(vec3 p, float r_a, float r_b, float s) {
    vec2 d = vec2(length(p.xz) - 2.0 * r_a + r_b, abs(p.y) - s);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - r_b;
}
float coneSDF(vec3 p, float s, float r1, float r2) {
    vec2 q = vec2(length(p.xz), p.y );

    vec2 k1 = vec2(r2, s);
    vec2 k2 = vec2(r2 - r1, 2.0 * s);
    vec2 ca = vec2(q.x - min(q.x, (q.y < 0.0)?r1:r2), abs(q.y) - s);
    vec2 cb = q - k1 + k2 * clamp(dot(k1 - q, k2) / dot2_2(k2), 0.0, 1.0);
    float s2 = (cb.x < 0.0 && ca.y < 0.0) ? -1.0 : 1.0;
    return s2*sqrt( min(dot2_2(ca),dot2_2(cb)) );
}
float roundedConeSDF(vec3 p, float s, float r1, float r2) {
    vec2 q = vec2( length(p.xz), p.y );

    float b = (r1 - r2) / s;
    float a = sqrt(1.0 - b * b);
    float k = dot(q, vec2(-b, a));

    if(k < 0.0)   return length(q) - r1;
    if(k > a * s) return length(q - vec2(0.0, s)) - r2;

    return dot(q, vec2(a,b) ) - r1;
}
float ellipsoidSDF(vec3 p, vec3 r) {
    float k0 = length(p / r);
    float k1 = length(p / (r * r));
    return k0*(k0 - 1.0) / k1;
}
float triSDF(vec3 p, vec3 a, vec3 b, vec3 c) {
    vec3 ba = b - a; vec3 pa = p - a;
    vec3 cb = c - b; vec3 pb = p - b;
    vec3 ac = a - c; vec3 pc = p - c;
    vec3 nor = cross( ba, ac );

    return sqrt(
    (sign(dot(cross(ba,nor),pa)) +
     sign(dot(cross(cb,nor),pb)) +
     sign(dot(cross(ac,nor),pc))<2.0)
     ?
     min( min(
     dot2_3(ba*clamp(dot(ba,pa)/dot2_3(ba),0.0,1.0)-pa),
     dot2_3(cb*clamp(dot(cb,pb)/dot2_3(cb),0.0,1.0)-pb) ),
     dot2_3(ac*clamp(dot(ac,pc)/dot2_3(ac),0.0,1.0)-pc) )
     :
     dot(nor,pa)*dot(nor,pa)/dot2_3(nor) );
}
float quadSDF(vec3 p, vec3 a, vec3 b, vec3 c, vec3 d) {
    vec3 ba = b - a; vec3 pa = p - a;
    vec3 cb = c - b; vec3 pb = p - b;
    vec3 dc = d - c; vec3 pc = p - c;
    vec3 ad = a - d; vec3 pd = p - d;
    vec3 nor = cross( ba, ad );

    return sqrt(
    (sign(dot(cross(ba,nor),pa)) +
     sign(dot(cross(cb,nor),pb)) +
     sign(dot(cross(dc,nor),pc)) +
     sign(dot(cross(ad,nor),pd))<3.0)
     ?
     min( min( min(
     dot2_3(ba*clamp(dot(ba,pa)/dot2_3(ba),0.0,1.0)-pa),
     dot2_3(cb*clamp(dot(cb,pb)/dot2_3(cb),0.0,1.0)-pb) ),
     dot2_3(dc*clamp(dot(dc,pc)/dot2_3(dc),0.0,1.0)-pc) ),
     dot2_3(ad*clamp(dot(ad,pd)/dot2_3(ad),0.0,1.0)-pd) )
     :
     dot(nor,pa)*dot(nor,pa)/dot2_3(nor) );
}
//Operators
/*vec4 opElongate(vec3 p, vec3 h) {
    //Not sure how to implement this
    //return vec4( p-clamp(p,-h,h), 0.0 ); // faster, but produces zero in the interior elongated box
    vec3 q = abs(p)-h;
    return vec4(min(max(q.x, max(q.y, q.z)), 0.0), max(q, 0.0));
}*/
vec4 opRound(vec3 p, float r) {
    return vec4(p, r);
}
float opOnion(float sdf_result, float thickness) {
    return abs(sdf_result) - thickness;
}
/*Maby latervec4 opExtrude(vec3 sdf_result, vec3 p, float h) {
    vec2 w = vec2(sdf_result, abs(p.z) - h);
  	return min(max(w.x,w.y),0.0) + length(max(w,0.0));
}
vec3 opRevolve(vec3 )*/
//Booleans
float opUnion(float d1, float d2) {
    return min(d1, d2);
}
float opSubtract(float d1, float d2) {
    return max(-d1, d2);
}
float opIntersect(float d1, float d2) {
    return max(d1, d2);
}
float opSmoothUnion(float d1, float d2, float k) {
    float h = clamp( 0.5 + 0.5*(d2-d1)/k, 0.0, 1.0 );
    return mix( d2, d1, h ) - k*h*(1.0-h);
}
float opSmoothSubtract(float d1, float d2, float k) {
    float h = clamp( 0.5 - 0.5*(d2+d1)/k, 0.0, 1.0 );
    return mix( d2, -d1, h ) + k*h*(1.0-h);
}
float opSmoothIntersect(float d1, float d2, float k) {
    float h = clamp( 0.5 - 0.5*(d2-d1)/k, 0.0, 1.0 );
    return mix( d2, d1, h ) + k*h*(1.0-h);
}
//Translations
vec3 translate(vec3 t, vec3 p) {
    return vec3(p.x + t.x,
                p.y + t.y,
                p.z + t.z);
}
vec3 rotateX(float t, vec3 p) {
    return vec3(p.x,
                cos(t)*p.y - sin(t)*p.z,
                sin(t)*p.y + cos(t)*p.z);
}
vec3 rotateY(float t, vec3 p) {
    return vec3(cos(t)*p.x + sin(t)*p.z,
                p.y,
                -sin(t)*p.x + cos(t)*p.z);
}
vec3 rotateZ(float t, vec3 p) {
    return vec3(cos(t)*p.x - sin(t)*p.y,
                sin(t)*p.x + cos(t)*p.y,
                p.z);
}
//Symmetry
vec3 symX(vec3 p) {
    p.x = abs(p.x);
    return p;
}
vec3 symY(vec3 p) {
    p.y = abs(p.y);
    return p;
}
vec3 symZ(vec3 p) {
    p.z = abs(p.z);
    return p;
}
//Patterns
vec3 patInfinite(vec3 p, vec3 o) {
    return mod(p, o) - 0.5*o;
}

float sceneSDF(vec3 p) {
    return boxSDF(translate(vec3(sin(time), 0.0, 0.0), rotateZ(sin(time), p)), vec3(1.0, 0.5, 0.5));
}

vec3 estimateNormal(vec3 p) {
    return normalize(vec3(
        sceneSDF(vec3(p.x + EPSILON, p.y, p.z)) - sceneSDF(vec3(p.x - EPSILON, p.y, p.z)),
        sceneSDF(vec3(p.x, p.y + EPSILON, p.z)) - sceneSDF(vec3(p.x, p.y - EPSILON, p.z)),
        sceneSDF(vec3(p.x, p.y, p.z  + EPSILON)) - sceneSDF(vec3(p.x, p.y, p.z - EPSILON))
    ));
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

vec3 rayDirection(float fieldOfView, vec2 s, vec2 fragCoord) {
    vec2 xy = fragCoord - s / 2.0;
    float z = s.y / tan(radians(fieldOfView) / 2.0);
    return normalize(vec3(xy, -z));
}

vec3 phongContribForLight(vec3 k_d, vec3 k_s, float alpha, vec3 p, vec3 eye,
                          vec3 lightPos, vec3 lightIntensity) {
    vec3 N = estimateNormal(p);
    vec3 L = normalize(lightPos - p);
    vec3 V = normalize(eye - p);
    vec3 R = normalize(reflect(-L, N));

    float dotLN = dot(L, N);
    float dotRV = dot(R, V);

    if (dotLN < 0.0) {
        // Light not visible from this point on the surface
        return vec3(0.0, 0.0, 0.0);
    }

    if (dotRV < 0.0) {
        // Light reflection in opposite direction as viewer, apply only diffuse
        // component
        return lightIntensity * (k_d * dotLN);
    }
    return lightIntensity * (k_d * dotLN + k_s * pow(dotRV, alpha));
}

/**
 * Lighting via Phong illumination.
 *
 * The vec3 returned is the RGB color of that point after lighting is applied.
 * k_a: Ambient color
 * k_d: Diffuse color
 * k_s: Specular color
 * alpha: Shininess coefficient
 * p: position of point being lit
 * eye: the position of the camera
 *
 * See https://en.wikipedia.org/wiki/Phong_reflection_model#Description
 */
vec3 phongIllumination(vec3 k_a, vec3 k_d, vec3 k_s, float alpha, vec3 p, vec3 eye) {
    const vec3 ambientLight = 0.5 * vec3(1.0, 1.0, 1.0);
    vec3 color = ambientLight * k_a;

    vec3 light1Pos = vec3(4.0 * sin(time),
                          2.0,
                          4.0 * cos(time));
    vec3 light1Intensity = vec3(0.4, 0.4, 0.4);

    color += phongContribForLight(k_d, k_s, alpha, p, eye,
                                  light1Pos,
                                  light1Intensity);

    vec3 light2Pos = vec3(2.0 * sin(0.37 * time),
                          2.0 * cos(0.37 * time),
                          2.0);
    vec3 light2Intensity = vec3(0.4, 0.4, 0.4);

    color += phongContribForLight(k_d, k_s, alpha, p, eye,
                                  light2Pos,
                                  light2Intensity);
    return color;
}

void main()
{
	vec3 dir = rayDirection(45.0, resolution/*vec2(900, 700)*/, gl_FragCoord.xy);
    vec3 eye = vec3(0.0, 0.0, 5.0);
    float dist = shortestDistanceToSurface(eye, dir, MIN_DIST, MAX_DIST);

    if (dist > MAX_DIST - EPSILON) {
        // Didn't hit anything
        Color = vec3(0.0, 0.0, 0.0);
		return;
    }

    // The closest point on the surface to the eyepoint along the view ray
    vec3 p = eye + dist * dir;

    vec3 K_a = vec3(0.2, 0.2, 0.2);
    vec3 K_d = vec3(0.7, 0.2, 0.2);
    vec3 K_s = vec3(1.0, 1.0, 1.0);
    float shininess = 10.0;

    Color = phongIllumination(K_a, K_d, K_s, shininess, p, eye);
}