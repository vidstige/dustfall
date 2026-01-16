#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContainerId(usize);

impl ContainerId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Volume(i64);

impl Volume {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn value(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// Amounts are in integer "moles" (amount-of-substance units), not mass.
pub struct Gas {
    // These amounts drive partial pressure when divided by volume.
    pub o2: i64,
    pub co2: i64,
    pub co: i64,
    pub h2o: i64,
}

impl Gas {
    pub fn zero() -> Self {
        Self {
            o2: 0,
            co2: 0,
            co: 0,
            h2o: 0,
        }
    }

    pub fn is_non_negative(&self) -> bool {
        self.o2 >= 0 && self.co2 >= 0 && self.co >= 0 && self.h2o >= 0
    }

    pub fn partial_pressure(amount: i64, volume: Volume) -> i64 {
        amount / volume.value()
    }

    pub fn pressure(&self, volume: Volume) -> i64 {
        Self::partial_pressure(self.o2, volume)
            + Self::partial_pressure(self.co2, volume)
            + Self::partial_pressure(self.co, volume)
            + Self::partial_pressure(self.h2o, volume)
    }

    pub fn can_apply_delta(&self, delta: Gas) -> bool {
        self.o2 + delta.o2 >= 0
            && self.co2 + delta.co2 >= 0
            && self.co + delta.co >= 0
            && self.h2o + delta.h2o >= 0
    }

    pub fn apply_delta(&mut self, delta: Gas) {
        self.o2 += delta.o2;
        self.co2 += delta.co2;
        self.co += delta.co;
        self.h2o += delta.h2o;
    }
}

pub fn gas_from_parts(
    volume: Volume,
    pressure: i64,
    o2_parts: i64,
    co2_parts: i64,
    h2o_parts: i64,
    divisor: i64,
) -> Gas {
    assert!(volume.value() > 0, "volume must be positive");
    assert!(pressure >= 0, "pressure must be non-negative");
    assert!(o2_parts >= 0, "o2_parts must be non-negative");
    assert!(co2_parts >= 0, "co2_parts must be non-negative");
    assert!(h2o_parts >= 0, "h2o_parts must be non-negative");
    assert!(divisor > 0, "divisor must be positive");

    let total = pressure * volume.value();
    let raw_o2 = total * o2_parts;
    let raw_co2 = total * co2_parts;
    let raw_h2o = total * h2o_parts;

    let o2 = raw_o2 / divisor;
    let co2 = raw_co2 / divisor;
    let h2o = raw_h2o / divisor;
    // Note: We floor each component, so the sum can be slightly below the intended total.

    Gas { o2, co2, co: 0, h2o }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fluid {
    pub h2o: i64,
}

impl Fluid {
    pub fn zero() -> Self {
        Self { h2o: 0 }
    }

    pub fn can_apply_delta(&self, delta: Fluid) -> bool {
        self.h2o + delta.h2o >= 0
    }

    pub fn apply_delta(&mut self, delta: Fluid) {
        self.h2o += delta.h2o;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Solid {
    pub ch2o: i64,
}

impl Solid {
    pub fn zero() -> Self {
        Self { ch2o: 0 }
    }

    pub fn can_apply_delta(&self, delta: Solid) -> bool {
        self.ch2o + delta.ch2o >= 0
    }

    pub fn apply_delta(&mut self, delta: Solid) {
        self.ch2o += delta.ch2o;
    }
}

#[derive(Debug, Clone)]
pub struct Container {
    volume: Volume,
    gas: Gas,
    fluid: Fluid,
    solid: Solid,
    children: Vec<ContainerId>,
}

impl Container {
    fn new(volume: Volume, gas: Gas, fluid: Fluid, solid: Solid) -> Self {
        Self {
            volume,
            gas,
            fluid,
            solid,
            children: Vec::new(),
        }
    }

    pub fn pressure(&self) -> i64 {
        self.gas.pressure(self.volume)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipe {
    pub a: ContainerId,
    pub b: ContainerId,
    // Flow rate per tick, expressed as moles of each gas.
    pub flow_rate: Gas,
}

impl Pipe {
    pub fn new(a: ContainerId, b: ContainerId, flow_rate: Gas) -> Self {
        assert!(flow_rate.is_non_negative(), "flow rates must be non-negative");
        Self { a, b, flow_rate }
    }
}

#[derive(Debug, Clone, Copy)]
struct Reaction {
    container: ContainerId,
    gas_delta: Gas,
    fluid_delta: Fluid,
    solid_delta: Solid,
}

impl Reaction {
    fn new(container: ContainerId, gas_delta: Gas, fluid_delta: Fluid, solid_delta: Solid) -> Self {
        Self {
            container,
            gas_delta,
            fluid_delta,
            solid_delta,
        }
    }

    fn check(&self) -> bool {
        let gas = self.gas_delta;
        let fluid = self.fluid_delta;
        let solid = self.solid_delta;

        let carbon = gas.co2 + gas.co + solid.ch2o;
        let hydrogen = 2 * (gas.h2o + fluid.h2o + solid.ch2o);
        let oxygen =
            2 * gas.o2 + 2 * gas.co2 + gas.co + gas.h2o + fluid.h2o + solid.ch2o;

        carbon == 0 && hydrogen == 0 && oxygen == 0
    }
}

#[derive(Debug)]
pub struct Engine {
    containers: Vec<Container>,
    pipes: Vec<Pipe>,
    reactions: Vec<Reaction>,
    root: ContainerId,
}

impl Engine {
    pub fn new(volume: Volume, gas: Gas, fluid: Fluid, solid: Solid) -> Self {
        let mut engine = Self {
            containers: Vec::new(),
            pipes: Vec::new(),
            reactions: Vec::new(),
            root: ContainerId(0),
        };
        let id = engine.insert_container(volume, gas, fluid, solid);
        engine.root = id;
        engine
    }

    pub fn root(&self) -> ContainerId {
        self.root
    }

    pub fn add_container(
        &mut self,
        parent: ContainerId,
        volume: Volume,
        gas: Gas,
        fluid: Fluid,
        solid: Solid,
    ) -> ContainerId {
        let id = self.insert_container(volume, gas, fluid, solid);
        self.containers[parent.index()].children.push(id);
        id
    }

    pub fn container(&self, id: ContainerId) -> &Container {
        &self.containers[id.index()]
    }

    pub fn container_mut(&mut self, id: ContainerId) -> &mut Container {
        &mut self.containers[id.index()]
    }

    pub fn pipes(&self) -> &[Pipe] {
        &self.pipes
    }

    pub fn add_pipe(&mut self, a: ContainerId, b: ContainerId, flow_rate: Gas) {
        assert!(a != b, "pipe endpoints must be different");
        self.pipes.push(Pipe::new(a, b, flow_rate));
    }

    pub fn add_reaction(
        &mut self,
        container: ContainerId,
        gas_delta: Gas,
        fluid_delta: Fluid,
        solid_delta: Solid,
    ) {
        // Use negative values to consume resources.
        let reaction = Reaction::new(container, gas_delta, fluid_delta, solid_delta);
        assert!(reaction.check(), "reaction is not atom-balanced");
        self.reactions.push(reaction);
    }

    pub fn tick(&mut self) {
        for reaction in self.reactions.iter().copied() {
            let container = &mut self.containers[reaction.container.index()];
            if !container.gas.can_apply_delta(reaction.gas_delta)
                || !container.fluid.can_apply_delta(reaction.fluid_delta)
                || !container.solid.can_apply_delta(reaction.solid_delta)
            {
                continue;
            }
            container.gas.apply_delta(reaction.gas_delta);
            container.fluid.apply_delta(reaction.fluid_delta);
            container.solid.apply_delta(reaction.solid_delta);
        }

        for pipe in self.pipes.clone() {
            self.apply_pipe_flow(pipe);
        }
    }

    fn insert_container(
        &mut self,
        volume: Volume,
        gas: Gas,
        fluid: Fluid,
        solid: Solid,
    ) -> ContainerId {
        let id = ContainerId(self.containers.len());
        self.containers
            .push(Container::new(volume, gas, fluid, solid));
        id
    }

    fn apply_pipe_flow(&mut self, pipe: Pipe) {
        let (a, b) = self.container_pair_mut(pipe.a, pipe.b);
        let mut delta = Gas::zero();
        let mut inverse = Gas::zero();

        let o2_flow = Self::flow_amount(
            a.gas.o2,
            a.volume,
            b.gas.o2,
            b.volume,
            pipe.flow_rate.o2,
        );
        let co2_flow = Self::flow_amount(
            a.gas.co2,
            a.volume,
            b.gas.co2,
            b.volume,
            pipe.flow_rate.co2,
        );
        let co_flow = Self::flow_amount(
            a.gas.co,
            a.volume,
            b.gas.co,
            b.volume,
            pipe.flow_rate.co,
        );
        let h2o_flow = Self::flow_amount(
            a.gas.h2o,
            a.volume,
            b.gas.h2o,
            b.volume,
            pipe.flow_rate.h2o,
        );

        delta.o2 = -o2_flow;
        delta.co2 = -co2_flow;
        delta.co = -co_flow;
        delta.h2o = -h2o_flow;

        inverse.o2 = o2_flow;
        inverse.co2 = co2_flow;
        inverse.co = co_flow;
        inverse.h2o = h2o_flow;

        a.gas.apply_delta(delta);
        b.gas.apply_delta(inverse);
    }

    fn container_pair_mut(
        &mut self,
        a: ContainerId,
        b: ContainerId,
    ) -> (&mut Container, &mut Container) {
        let a_idx = a.index();
        let b_idx = b.index();
        assert!(a_idx != b_idx, "container pair must be different");
        if a_idx < b_idx {
            let (left, right) = self.containers.split_at_mut(b_idx);
            (&mut left[a_idx], &mut right[0])
        } else {
            let (left, right) = self.containers.split_at_mut(a_idx);
            (&mut right[0], &mut left[b_idx])
        }
    }

    fn flow_amount(
        amount_a: i64,
        volume_a: Volume,
        amount_b: i64,
        volume_b: Volume,
        max_flow: i64,
    ) -> i64 {
        let va = volume_a.value() as i128;
        let vb = volume_b.value() as i128;
        let numerator = (amount_a as i128) * vb - (amount_b as i128) * va;
        let denom = va + vb;
        if denom <= 0 {
            return 0;
        }
        let mut equalize = numerator / denom;
        let max_flow = max_flow as i128;
        let hi = (amount_a as i128).min(max_flow);
        let lo = -(amount_b as i128).min(max_flow);
        if equalize > hi {
            equalize = hi;
        } else if equalize < lo {
            equalize = lo;
        }
        equalize as i64
    }
}

pub fn add_human(engine: &mut Engine, container: ContainerId, o2_per_tick: i64) {
    assert!(o2_per_tick >= 0, "o2_per_tick must be non-negative");
    engine.add_reaction(
        container,
        Gas {
            o2: -o2_per_tick,
            co2: o2_per_tick,
            co: 0,
            h2o: o2_per_tick,
        },
        Fluid::zero(),
        Solid { ch2o: -o2_per_tick },
    );
}

pub fn add_photosynthesis(
    engine: &mut Engine,
    container: ContainerId,
    co2_per_tick: i64,
) {
    assert!(co2_per_tick >= 0, "co2_per_tick must be non-negative");
    engine.add_reaction(
        container,
        Gas {
            o2: co2_per_tick,
            co2: -co2_per_tick,
            co: 0,
            h2o: 0,
        },
        Fluid { h2o: -co2_per_tick },
        Solid { ch2o: co2_per_tick },
    );
}

pub fn add_moxie(engine: &mut Engine, container: ContainerId, co2_per_tick: i64) {
    assert!(co2_per_tick >= 0, "co2_per_tick must be non-negative");
    assert!(
        co2_per_tick % 2 == 0,
        "co2_per_tick must be even for 2 CO2 -> 2 CO + O2"
    );
    let o2_per_tick = co2_per_tick / 2;
    engine.add_reaction(
        container,
        Gas {
            o2: o2_per_tick,
            co2: -co2_per_tick,
            co: co2_per_tick,
            h2o: 0,
        },
        Fluid::zero(),
        Solid::zero(),
    );
}
