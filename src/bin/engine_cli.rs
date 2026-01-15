use dustfall::engine::{
    add_human, add_moxie, add_photosynthesis, gas_from_parts, Engine, Fluid, Gas, Solid, Volume,
};
use dustfall::units::PressureScale;

fn thin_atmosphere(volume: Volume, pressure: i64) -> Gas {
    // The reported composition is a volume (molar) ratio, so we treat it as mole fractions.
    const DIVISOR: i64 = 10_000;
    const CO2_PARTS: i64 = 9_532;
    const O2_PARTS: i64 = 13;
    gas_from_parts(volume, pressure, O2_PARTS, CO2_PARTS, 0, DIVISOR)
}

fn main() {
    let ticks: usize = std::env::args()
        .nth(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(10);

    let scale = PressureScale::new(100.0);
    let atmosphere_volume = Volume::new(93_000_000_000_000);
    let mut engine = Engine::new(
        atmosphere_volume,
        thin_atmosphere(atmosphere_volume, scale.pressure_for_parts(800.0)),
        Fluid::zero(),
        Solid::zero(),
    );
    let root = engine.root();
    let habitat = engine.add_container(
        root,
        Volume::new(100),
        Gas {
            o2: 20_200,
            co2: 80_800,
            co: 0,
            h2o: 0,
        },
        Fluid::zero(),
        Solid { ch2o: 500 },
    );
    add_human(&mut engine, habitat, 3);
    add_photosynthesis(&mut engine, habitat, 2);
    add_moxie(&mut engine, habitat, 2);

    for tick in 0..ticks {
        println!(
            "tick {}: atmosphere={} Pa, habitat={} Pa",
            tick,
            scale.to_pascal(engine.container(root).pressure()),
            scale.to_pascal(engine.container(habitat).pressure())
        );
        engine.tick();
    }
}
