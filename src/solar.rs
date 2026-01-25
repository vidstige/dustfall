use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct PlanetParameters {
    pub sol_seconds: f32, // Length of a mean solar day, in seconds.
    pub year_days: f32, // Orbital period in Earth days.
    pub axial_tilt: f32, // Obliquity in radians.
}

impl PlanetParameters {
    pub fn solar_longitude(&self, time_seconds: f32) -> f32 {
        let days_since_epoch = time_seconds / 86_400.0;
        let mean_motion = TAU / self.year_days;
        (days_since_epoch * mean_motion).rem_euclid(TAU)
    }

    pub fn solar_declination(&self, time_seconds: f32) -> f32 {
        let ls = self.solar_longitude(time_seconds);
        (self.axial_tilt.sin() * ls.sin()).asin()
    }

    pub fn local_solar_fraction(&self, time_seconds: f32, longitude: f32) -> f32 {
        let sols_since_epoch = time_seconds / self.sol_seconds;
        let prime_meridian = sols_since_epoch.rem_euclid(1.0);
        (prime_meridian + longitude / TAU).rem_euclid(1.0)
    }

    pub fn local_mean_solar_time_hours(&self, time_seconds: f32, longitude: f32) -> f32 {
        self.local_solar_fraction(time_seconds, longitude) * 24.0
    }
}

pub const MARS: PlanetParameters = PlanetParameters {
    sol_seconds: 88_775.244,
    year_days: 686.971,
    axial_tilt: deg_to_rad(25.19),
};

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub latitude: f32, // Latitude in radians.
    pub longitude: f32, // Longitude in radians (east-positive).
}

// time+location -> sun direction
pub fn solar_direction(
    params: &PlanetParameters,
    location: Location,
    time_seconds: f32,
) -> (f32, f32, f32) {
    let lat = location.latitude;
    let declination = params.solar_declination(time_seconds);
    let local_fraction = params.local_solar_fraction(time_seconds, location.longitude);
    let local_time_angle = (local_fraction - 0.5) * TAU;

    let east = declination.cos() * local_time_angle.sin();
    let north =
        lat.cos() * declination.sin() - lat.sin() * declination.cos() * local_time_angle.cos();
    let up = lat.sin() * declination.sin() + lat.cos() * declination.cos() * local_time_angle.cos();

    let (east, north, up) = normalize((east, north, up));
    (east, up, north)
}

fn normalize((x, y, z): (f32, f32, f32)) -> (f32, f32, f32) {
    let len = (x * x + y * y + z * z).sqrt();
    if len == 0.0 {
        return (x, y, z);
    }
    (x / len, y / len, z / len)
}

const fn deg_to_rad(value: f32) -> f32 {
    value * (TAU / 360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCATION: Location = Location {
        latitude: deg_to_rad(54.0),
        longitude: deg_to_rad(137.4),
    };

    fn dot(a: (f32, f32, f32), b: (f32, f32, f32)) -> f32 {
        a.0 * b.0 + a.1 * b.1 + a.2 * b.2
    }

    #[test]
    fn solar_direction_is_normalized() {
        let (x, y, z) = solar_direction(&MARS, LOCATION, 1_704_110_400.0);
        let len = (x * x + y * y + z * z).sqrt();
        assert!((len - 1.0).abs() < 1e-3, "len={len}");
    }

    #[test]
    fn local_time_progresses_through_day() {
        let base = 1_704_067_200.0;
        let later = base + 6.0 * 3600.0;
        let lmst0 = MARS.local_mean_solar_time_hours(base, LOCATION.longitude);
        let lmst1 = MARS.local_mean_solar_time_hours(later, LOCATION.longitude);
        let delta = (lmst1 - lmst0 + 24.0).rem_euclid(24.0);
        assert!((5.0..7.0).contains(&delta), "delta={delta}");
    }

    #[test]
    fn solar_direction_varies_over_half_day() {
        let base = 1_704_067_200.0;
        let later = base + 12.0 * 3600.0;
        let a = solar_direction(&MARS, LOCATION, base);
        let b = solar_direction(&MARS, LOCATION, later);
        assert!(dot(a, b) < -0.2, "dot={}", dot(a, b));
    }

    #[test]
    fn solar_direction_repeats_each_sol() {
        let base = 1_704_110_400.0;
        let next_sol = base + MARS.sol_seconds;
        let a = solar_direction(&MARS, LOCATION, base);
        let b = solar_direction(&MARS, LOCATION, next_sol);
        assert!(dot(a, b) > 0.999, "dot={}", dot(a, b));
    }
}
