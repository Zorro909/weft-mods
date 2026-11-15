//! The light and air of a film's shots: fog, sky, sunlight, clouds and how sound carries, set
//! with `Film::weather` and lifted with `Film::clear_weather` or at the film's end. It changes
//! only what the watching player sees and hears, not the world's own weather or time.

/// What a film's weather changes; everything left out stays as the world has it.
#[derive(Clone, Debug, PartialEq)]
pub struct Weather {
    pub fog: Option<Fog>,
    /// The sky's colour, `0xRRGGBB`.
    pub sky: Option<u32>,
    /// Light from the sky and sun, 0 (none) to 1 (full day).
    pub sunlight: Option<f32>,
    /// How much of the clouds shows, 0 to 1.
    pub clouds: Option<f32>,
    /// The clouds' colour before lighting, `0xRRGGBB`.
    pub cloud_color: Option<u32>,
    pub acoustics: Option<Acoustics>,
    /// Seconds of film it takes to come in, and to go again.
    pub fade: f32,
}

/// Fog of `color` (`0xRRGGBB`) from `start` blocks away, hiding everything past `end`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fog {
    pub color: u32,
    pub start: f32,
    pub end: f32,
}

/// How every sound is heard: muffled above `cutoff_hz`, with `reverb` (0 none) and at `volume`
/// (1 as it is).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Acoustics {
    pub cutoff_hz: f32,
    pub reverb: f32,
    pub volume: f32,
}

/// Seconds a weather takes to come in unless told.
const FADE: f32 = 2.0;

impl Default for Weather {
    fn default() -> Weather {
        Weather {
            fog: None,
            sky: None,
            sunlight: None,
            clouds: None,
            cloud_color: None,
            acoustics: None,
            fade: FADE,
        }
    }
}

impl Weather {
    /// A weather that changes nothing yet, coming in over 2 seconds.
    pub fn new() -> Weather {
        Weather::default()
    }

    /// Evening light: a low, orange sky, warm haze and dimmed sunlight.
    pub fn dusk() -> Weather {
        Weather::new()
            .fog(0xC07850, 48.0, 220.0)
            .sky(0x6A5878)
            .sunlight(0.45)
            .cloud_color(0xFF9A60)
    }

    /// A grey mist that hides everything past about 40 blocks, and no clouds.
    pub fn mist() -> Weather {
        Weather::new()
            .fog(0xA8B0B4, 2.0, 40.0)
            .sky(0xA8B0B4)
            .sunlight(0.7)
            .clouds(0.0)
    }

    /// Under the ground: no sky light, black fog closing in at 32 blocks, and echoing sound.
    pub fn underground() -> Weather {
        Weather::new()
            .fog(0x000000, 4.0, 32.0)
            .sky(0x000000)
            .sunlight(0.0)
            .clouds(0.0)
            .acoustics(8000.0, 0.6, 1.0)
    }

    pub fn fog(mut self, color: u32, start: f32, end: f32) -> Weather {
        self.fog = Some(Fog { color, start, end });
        self
    }

    pub fn sky(mut self, color: u32) -> Weather {
        self.sky = Some(color);
        self
    }

    pub fn sunlight(mut self, strength: f32) -> Weather {
        self.sunlight = Some(strength);
        self
    }

    pub fn clouds(mut self, cover: f32) -> Weather {
        self.clouds = Some(cover);
        self
    }

    pub fn cloud_color(mut self, color: u32) -> Weather {
        self.cloud_color = Some(color);
        self
    }

    pub fn acoustics(mut self, cutoff_hz: f32, reverb: f32, volume: f32) -> Weather {
        self.acoustics = Some(Acoustics {
            cutoff_hz,
            reverb,
            volume,
        });
        self
    }

    pub fn fade(mut self, secs: f32) -> Weather {
        self.fade = secs;
        self
    }
}
