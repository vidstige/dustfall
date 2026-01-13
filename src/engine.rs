#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContainerId(usize);

impl ContainerId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Volume(i32);

impl Volume {
    pub fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn value(self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// Amounts are in integer "moles" (amount-of-substance units), not mass.
pub struct Gas {
    // These amounts drive partial pressure when divided by volume.
    pub o2: i32,
    pub co2: i32,
    pub co: i32,
}

impl Gas {
    pub fn partial_pressure(amount: i32, volume: Volume) -> i32 {
        amount / volume.value()
    }

    pub fn pressure(&self, volume: Volume) -> i32 {
        Self::partial_pressure(self.o2, volume)
            + Self::partial_pressure(self.co2, volume)
            + Self::partial_pressure(self.co, volume)
    }

    pub fn can_apply_delta(&self, delta: Gas) -> bool {
        self.o2 + delta.o2 >= 0 && self.co2 + delta.co2 >= 0 && self.co + delta.co >= 0
    }

    pub fn apply_delta(&mut self, delta: Gas) {
        self.o2 += delta.o2;
        self.co2 += delta.co2;
        self.co += delta.co;
    }
}

#[derive(Debug, Clone)]
pub struct Container {
    volume: Volume,
    gas: Gas,
    children: Vec<ContainerId>,
}

impl Container {
    fn new(volume: Volume, gas: Gas) -> Self {
        Self {
            volume,
            gas,
            children: Vec::new(),
        }
    }

    pub fn volume(&self) -> Volume {
        self.volume
    }

    pub fn gas(&self) -> Gas {
        self.gas
    }

    pub fn gas_mut(&mut self) -> &mut Gas {
        &mut self.gas
    }

    pub fn pressure(&self) -> i32 {
        self.gas.pressure(self.volume)
    }

    pub fn children(&self) -> &[ContainerId] {
        &self.children
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipe {
    pub a: ContainerId,
    pub b: ContainerId,
}

impl Pipe {
    pub fn new(a: ContainerId, b: ContainerId) -> Self {
        Self { a, b }
    }
}

#[derive(Debug, Clone, Copy)]
struct Reaction {
    container: ContainerId,
    delta: Gas,
}

impl Reaction {
    fn new(container: ContainerId, delta: Gas) -> Self {
        Self { container, delta }
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
    pub fn new(volume: Volume, gas: Gas) -> Self {
        let mut engine = Self {
            containers: Vec::new(),
            pipes: Vec::new(),
            reactions: Vec::new(),
            root: ContainerId(0),
        };
        let id = engine.insert_container(volume, gas);
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
    ) -> ContainerId {
        let id = self.insert_container(volume, gas);
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

    pub fn add_pipe(&mut self, a: ContainerId, b: ContainerId) {
        self.pipes.push(Pipe::new(a, b));
    }

    pub fn add_reaction(
        &mut self,
        container: ContainerId,
        delta: Gas,
    ) {
        // Use negative values to consume gases.
        self.reactions.push(Reaction::new(container, delta));
    }

    pub fn tick(&mut self) {
        for reaction in self.reactions.iter().copied() {
            let container = &mut self.containers[reaction.container.index()];
            if !container.gas.can_apply_delta(reaction.delta) {
                continue;
            }
            container.gas.apply_delta(reaction.delta);
        }
    }

    fn insert_container(&mut self, volume: Volume, gas: Gas) -> ContainerId {
        let id = ContainerId(self.containers.len());
        self.containers.push(Container::new(volume, gas));
        id
    }
}

pub fn add_human(engine: &mut Engine, container: ContainerId, o2_per_tick: i32) {
    assert!(o2_per_tick >= 0, "o2_per_tick must be non-negative");
    engine.add_reaction(
        container,
        Gas {
            o2: -o2_per_tick,
            co2: o2_per_tick,
            co: 0,
        },
    );
}

pub fn add_moxie(
    engine: &mut Engine,
    container: ContainerId,
    co2_per_tick: i32,
    o2_per_tick: i32,
    co_per_tick: i32,
) {
    assert!(co2_per_tick >= 0, "co2_per_tick must be non-negative");
    assert!(o2_per_tick >= 0, "o2_per_tick must be non-negative");
    assert!(co_per_tick >= 0, "co_per_tick must be non-negative");
    engine.add_reaction(
        container,
        Gas {
            o2: o2_per_tick,
            co2: -co2_per_tick,
            co: co_per_tick,
        },
    );
}
