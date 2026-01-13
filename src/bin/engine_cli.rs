use dustfall::engine::{
    add_human, add_moxie, add_photosynthesis, gas_from_parts, Engine, Fluid, Gas, Solid, Volume,
};
use dustfall::units::PressureScale;

fn thin_atmosphere(volume: Volume, pressure: i32) -> Gas {
    // The reported composition is a volume (molar) ratio, so we treat it as mole fractions.
    const DIVISOR: i32 = 10_000;
    const CO2_PARTS: i32 = 9_532;
    const O2_PARTS: i32 = 13;
    gas_from_parts(volume, pressure, O2_PARTS, CO2_PARTS, 0, DIVISOR)
}

fn main() {
    let ticks: usize = std::env::args()
        .nth(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(10);

    let scale = PressureScale::new(100.0);
    let mut engine = Engine::new(
        Volume::new(1000),
        thin_atmosphere(Volume::new(1000), scale.pressure_for_parts(800.0)),
        Fluid { h2o: 0 },
        Solid { ch2o: 0 },
    );
    let root = engine.root();
    let habitat = engine.add_container(
        root,
        Volume::new(100),
        Gas {
            o2: 2000,
            co2: 8000,
            co: 0,
            h2o: 0,
        },
        Fluid { h2o: 0 },
        Solid { ch2o: 500 },
    );
    add_human(&mut engine, habitat, 3);
    add_photosynthesis(&mut engine, habitat, 2);
    add_moxie(&mut engine, habitat, 2);

    println!("tick 0: {:?}", engine.container(habitat));
    for tick in 1..=ticks {
        engine.tick();
        println!("tick {}: {:?}", tick, engine.container(habitat));
    }
}
