// A golden-hour grade: more colour, more red and less blue, mixed in by strength.
fn apply(p: PostPixel) -> vec4<f32> {
    let c = p.color.rgb;
    let grey = vec3(dot(c, vec3(0.2126, 0.7152, 0.0722)));
    let graded = mix(grey, c, 1.15) * vec3(1.12, 1.0, 0.8);
    return vec4(mix(c, graded, p.strength), 1.0);
}
