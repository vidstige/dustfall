use std::f32::consts::TAU;
use std::time::{SystemTime, UNIX_EPOCH};

const SELECTED_LATITUDE: f32 = deg_to_rad(54.0);
const SELECTED_LONGITUDE: f32 = deg_to_rad(137.4);

struct PlanetParameters {
    sol_seconds: f32, // Length of a mean solar day, in seconds.
    year_days: f32, // Orbital period in Earth days.
    axial_tilt: f32, // Obliquity in radians.
}

impl PlanetParameters {
    fn solar_longitude(&self, unix_seconds: i64) -> f32 {
        let days_since_epoch = unix_seconds as f32 / 86_400.0;
        let mean_motion = TAU / self.year_days;
        (days_since_epoch * mean_motion).rem_euclid(TAU)
    }

    fn solar_declination(&self, unix_seconds: i64) -> f32 {
        let ls = self.solar_longitude(unix_seconds);
        (self.axial_tilt.sin() * ls.sin()).asin()
    }

    fn local_solar_fraction(&self, unix_seconds: i64, longitude: f32) -> f32 {
        let sols_since_epoch = unix_seconds as f32 / self.sol_seconds;
        let prime_meridian = sols_since_epoch.rem_euclid(1.0);
        (prime_meridian + longitude / TAU).rem_euclid(1.0)
    }
}

const MARS: PlanetParameters = PlanetParameters {
    sol_seconds: 88_775.244,
    year_days: 686.971,
    axial_tilt: deg_to_rad(25.19),
};

#[derive(Debug, Clone, Copy)]
struct Location {
    latitude: f32, // Latitude in radians.
    longitude: f32, // Longitude in radians (east-positive).
}

// time+location -> sun direction
fn solar_direction(
    params: &PlanetParameters,
    location: Location,
    unix_seconds: i64,
) -> (f32, f32, f32) {
    let lat = location.latitude;
    let declination = params.solar_declination(unix_seconds);
    let local_fraction = params.local_solar_fraction(unix_seconds, location.longitude);
    let local_time_angle = (local_fraction - 0.5) * TAU;

    let east = declination.cos() * local_time_angle.sin();
    let north =
        lat.cos() * declination.sin() - lat.sin() * declination.cos() * local_time_angle.cos();
    let up = lat.sin() * declination.sin() + lat.cos() * declination.cos() * local_time_angle.cos();

    normalize((east, north, up))
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

// time & other parsing
fn parse_args(args: &[String]) -> Result<(Location, i64), String> {
    let mut location = Location {
        latitude: SELECTED_LATITUDE,
        longitude: SELECTED_LONGITUDE,
    };
    let unix_seconds = match args.len() {
        0 => unix_seconds_now(),
        1 => return Err("Expected latitude and longitude if any coordinate is provided.".into()),
        2 => {
            location.latitude = parse_f32("latitude", &args[0])?;
            location.longitude = parse_f32("longitude", &args[1])?;
            unix_seconds_now()
        }
        3 => {
            location.latitude = parse_f32("latitude", &args[0])?;
            location.longitude = parse_f32("longitude", &args[1])?;
            parse_unix_seconds(&args[2])?
        }
        _ => return Err("Too many arguments.".into()),
    };

    Ok((location, unix_seconds))
}

fn print_usage() {
    eprintln!(
        "Usage: mars_sun_cli [lat lon [time]]\n\
         Default location is {:.4}, {:.4} (about 60% from equator to north pole).\n\
         time must be unix seconds.",
        SELECTED_LATITUDE, SELECTED_LONGITUDE
    );
}

fn parse_f32(label: &str, value: &str) -> Result<f32, String> {
    value
        .parse::<f32>()
        .map_err(|_| format!("Invalid {label}: {value}"))
}

fn parse_unix_seconds(value: &str) -> Result<i64, String> {
    value
        .parse::<i64>()
        .map_err(|_| format!("Invalid time '{value}', expected unix seconds."))
}

fn unix_seconds_now() -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch");
    duration.as_secs() as i64
}

// the command line main function
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (location, unix_seconds) = match parse_args(&args) {
        Ok(values) => values,
        Err(message) => {
            eprintln!("{message}");
            print_usage();
            return;
        }
    };

    let (x, y, z) = solar_direction(&MARS, location, unix_seconds);

    println!(
        "Selected Mars location: lat={:.4}, lon={:.4}",
        location.latitude, location.longitude
    );
    println!("Unix seconds: {unix_seconds}");
    println!("World axes: +X east, +Y up, +Z north at the reference location.");
    println!("Sun direction (x, y, z): [{:.4}, {:.4}, {:.4}]", x, y, z);
}
