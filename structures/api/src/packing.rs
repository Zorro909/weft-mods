//! Palette indices packed into `u64` words with 4, 8 or 16 bits each, the way a world save
//! stores a cube's blocks: lowest bits first, and an index never straddles two words. A palette
//! of one state needs no words at all, written as 0 bits.

/// Bits per index for a palette of `len` states.
fn bits_for(len: usize) -> u32 {
    match len {
        0..=1 => 0,
        2..=16 => 4,
        17..=256 => 8,
        _ => 16,
    }
}

/// Appends the width byte, then the words.
pub fn write(out: &mut Vec<u8>, indices: &[u16], palette_len: usize) {
    let bits = bits_for(palette_len);
    out.push(bits as u8);
    if bits == 0 {
        return;
    }
    for group in indices.chunks((64 / bits) as usize) {
        let word = group
            .iter()
            .enumerate()
            .fold(0u64, |word, (k, &i)| word | (i as u64) << (k as u32 * bits));
        out.extend_from_slice(&word.to_le_bytes());
    }
}

/// `volume` indices from what `write` wrote, `None` unless each is inside the palette.
pub fn read(data: &[u8], volume: usize, palette_len: usize) -> Option<Vec<u16>> {
    let (&bits, words) = data.split_first()?;
    let bits = bits as u32;
    if bits == 0 {
        return (palette_len > 0 || volume == 0).then(|| vec![0; volume]);
    }
    if !matches!(bits, 4 | 8 | 16) {
        return None;
    }
    let per = (64 / bits) as usize;
    let words = words.get(..volume.div_ceil(per) * 8)?;
    let mask = (1u64 << bits) - 1;
    let indices: Vec<u16> = words
        .chunks_exact(8)
        .flat_map(|word| {
            let word = u64::from_le_bytes(word.try_into().unwrap());
            (0..per).map(move |k| (word >> (k as u32 * bits) & mask) as u16)
        })
        .take(volume)
        .collect();
    indices
        .iter()
        .all(|&i| (i as usize) < palette_len)
        .then_some(indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(indices: &[u16], palette_len: usize) -> Option<Vec<u16>> {
        let mut out = Vec::new();
        write(&mut out, indices, palette_len);
        read(&out, indices.len(), palette_len)
    }

    #[test]
    fn every_width_reads_back() {
        for palette_len in [1, 2, 16, 17, 256, 257, 4000] {
            let indices: Vec<u16> = (0..1000).map(|i| (i * 7 % palette_len) as u16).collect();
            assert_eq!(
                round_trip(&indices, palette_len).as_deref(),
                Some(&indices[..]),
                "palette of {palette_len}"
            );
        }
    }

    #[test]
    fn one_state_takes_no_words() {
        let mut out = Vec::new();
        write(&mut out, &[0; 500], 1);
        assert_eq!(out, [0], "only the width byte");
    }

    #[test]
    fn indices_outside_the_palette_fail() {
        let mut out = Vec::new();
        write(&mut out, &[0, 1, 5], 6);
        assert_eq!(read(&out, 3, 3), None, "5 is past a palette of 3");
    }

    #[test]
    fn truncated_words_fail() {
        let mut out = Vec::new();
        write(&mut out, &[1; 40], 2);
        out.pop();
        assert_eq!(read(&out, 40, 2), None, "the last word is cut short");
    }

    #[test]
    fn unknown_widths_fail() {
        assert_eq!(
            read(&[3, 0, 0, 0, 0, 0, 0, 0, 0], 4, 2),
            None,
            "3 bits is not a width"
        );
    }
}
