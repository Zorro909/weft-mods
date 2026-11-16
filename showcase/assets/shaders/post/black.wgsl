// A film fading to black: none of the picture is left at full strength.
fn apply(p: PostPixel) -> vec4<f32> {
    return vec4(p.color.rgb * (1.0 - p.strength), 1.0);
}
