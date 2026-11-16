// A moonlit grade: less colour, less red and more blue, mixed in by strength.
fn apply(p: PostPixel) -> vec4<f32> {
    let c = p.color.rgb;
    let grey = vec3(dot(c, vec3(0.2126, 0.7152, 0.0722)));
    let graded = mix(grey, c, 0.8) * vec3(0.85, 0.97, 1.15);
    return vec4(mix(c, graded, p.strength), 1.0);
}
