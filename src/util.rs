use num_traits::{FromPrimitive, NumCast, ToPrimitive};

pub fn hue_to_rgb(hue: f64) -> [f32; 4] {
    let chroma = 1.0; // Chroma is the difference between the maximum and minimum RGB components
    let hue_prime = hue / 60.0; 
    let x = chroma * (1.0 - ((hue_prime % 2.0) - 1.0).abs());

    let (r1, g1, b1) = if hue_prime <= 1.0 {
        (chroma, x, 0.0)
    } else if hue_prime <= 2.0 {
        (x, chroma, 0.0)
    } else if hue_prime <= 3.0 {
        (0.0, chroma, x)
    } else if hue_prime <= 4.0 {
        (0.0, x, chroma)
    } else if hue_prime <= 5.0 {
        (x, 0.0, chroma)
    } else {
        (chroma, 0.0, x)
    };

    // RGB are adjusted by adding the same amount to each component 
    // to match lightness. This example uses fixed lightness and no adjustment.
    let m = 0.0; // m is the lightness adjustment factor
    let (r, g, b) = (
        (r1 + m), 
        (g1 + m), 
        (b1 + m),
    );

    return [r as f32, g as f32, b as f32, 1.0];
}

pub fn linspace<T>(start: T, end: T, num_points: usize) -> Vec<T>
where
    T: NumCast + Copy + FromPrimitive,
{
    if num_points == 0 {
        return Vec::new();
    } else if num_points == 1 {
        return vec![start];
    }

    let start_f64 = ToPrimitive::to_f64(&start).unwrap();
    let end_f64 = ToPrimitive::to_f64(&end).unwrap();
    let step = (end_f64 - start_f64) / ((num_points - 1) as f64);

    (0..num_points)
        .map(|i| {
            FromPrimitive::from_f64(start_f64 + step * (i as f64)).unwrap()
        })
        .collect()
}