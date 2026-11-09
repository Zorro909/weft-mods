//! Reading a sketch file: one thing a line, `#` starting a comment, points as `x,z`.
//!
//! ```text
//! path   3  10,40 30,38 52,44      a gravel path 3 wide, curving through these points
//! track     52,44 80,60            a cart track: two ruts
//! street 5  0,20 40,22 70,30       a cobbled street, paving 5 wide between kerbs
//! square    40,10 60,8 64,30 42,34 a paved square: the area inside these corners
//! field     ..corners..            strips of wheat
//! garden    ..corners..            rows of young wheat between furrows
//! hedge     ..points..             a line of leaves
//! place  crofter_cottage right 20,30 [styles]   a structure: its turn and middle
//! view   green 30,2,50 40,0,30     a shot named green: eye and target, y above the ground
//! ```

/// Everything a sketch holds.
#[derive(Default)]
pub struct Sketch {
    pub features: Vec<Feature>,
    pub places: Vec<Place>,
    pub views: Vec<View>,
}

/// What is drawn on the ground, in the sketch's blocks.
pub enum Feature {
    Path { width: f32, line: Vec<[f32; 2]> },
    Track { line: Vec<[f32; 2]> },
    Street { width: f32, line: Vec<[f32; 2]> },
    Square { area: Vec<[f32; 2]> },
    Field { area: Vec<[f32; 2]> },
    Garden { area: Vec<[f32; 2]> },
    Hedge { line: Vec<[f32; 2]> },
}

/// A structure of this mod, by the middle of its footprint as turned.
pub struct Place {
    pub name: String,
    pub turn: String,
    pub middle: [i32; 2],
    pub styles: String,
}

pub struct View {
    pub name: String,
    /// x, height above the ground, z.
    pub eye: [f32; 3],
    pub target: [f32; 3],
}

pub fn read(file: &str) -> Sketch {
    let text =
        std::fs::read_to_string(file).unwrap_or_else(|e| panic!("could not read {file}: {e}"));
    let mut sketch = Sketch::default();
    for (number, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if !line.is_empty() {
            let at = || format!("{file}:{}: {line:?}", number + 1);
            add(&mut sketch, line, &at);
        }
    }
    sketch
}

fn add(sketch: &mut Sketch, line: &str, at: &dyn Fn() -> String) {
    let words: Vec<&str> = line.split_whitespace().collect();
    let points = |from: usize| {
        words[from..]
            .iter()
            .map(|w| point(w, at))
            .collect::<Vec<_>>()
    };
    let width = || {
        words
            .get(1)
            .and_then(|w| w.parse().ok())
            .unwrap_or_else(|| panic!("{}: no width", at()))
    };
    let feature = match words[0] {
        "path" => Feature::Path {
            width: width(),
            line: points(2),
        },
        "track" => Feature::Track { line: points(1) },
        "street" => Feature::Street {
            width: width(),
            line: points(2),
        },
        "square" => Feature::Square { area: points(1) },
        "field" => Feature::Field { area: points(1) },
        "garden" => Feature::Garden { area: points(1) },
        "hedge" => Feature::Hedge { line: points(1) },
        "place" => return sketch.places.push(place(&words, at)),
        "view" => return sketch.views.push(view(&words, at)),
        other => panic!("{}: nothing is called {other}", at()),
    };
    sketch.features.push(feature);
}

fn place(words: &[&str], at: &dyn Fn() -> String) -> Place {
    let [_, name, turn, middle, rest @ ..] = words else {
        panic!("{}: not `place name turn x,z [styles]`", at());
    };
    let [x, z] = point(middle, at);
    Place {
        name: name.to_string(),
        turn: turn.to_string(),
        middle: [x as i32, z as i32],
        styles: rest.join(","),
    }
}

fn view(words: &[&str], at: &dyn Fn() -> String) -> View {
    let [_, name, eye, target] = words else {
        panic!("{}: not `view name x,y,z x,y,z`", at());
    };
    View {
        name: name.to_string(),
        eye: triple(eye, at),
        target: triple(target, at),
    }
}

fn point(word: &str, at: &dyn Fn() -> String) -> [f32; 2] {
    let numbers = numbers(word, at);
    let [x, z] = numbers[..] else {
        panic!("{}: {word} is not x,z", at());
    };
    [x, z]
}

fn triple(word: &str, at: &dyn Fn() -> String) -> [f32; 3] {
    let numbers = numbers(word, at);
    let [x, y, z] = numbers[..] else {
        panic!("{}: {word} is not x,y,z", at());
    };
    [x, y, z]
}

fn numbers(word: &str, at: &dyn Fn() -> String) -> Vec<f32> {
    let parse = |n: &str| {
        n.parse()
            .unwrap_or_else(|_| panic!("{}: {n} is no number", at()))
    };
    word.split(',').map(parse).collect()
}
