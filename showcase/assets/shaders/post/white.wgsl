// A film fading to white: all white at full strength.
fn apply(p: PostPixel) -> vec4<f32> {
    return vec4(mix(p.color.rgb, vec3(1.0), p.strength), 1.0);
}
