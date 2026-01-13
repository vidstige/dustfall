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

    pub fn can_consume(&self, other: Gas) -> bool {
        self.o2 >= other.o2 && self.co2 >= other.co2 && self.co >= other.co
    }

    pub fn consume(&mut self, other: Gas) {
        self.o2 -= other.o2;
        self.co2 -= other.co2;
        self.co -= other.co;
    }

    pub fn produce(&mut self, other: Gas) {
        self.o2 += other.o2;
        self.co2 += other.co2;
        self.co += other.co;
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
    consumes: Gas,
    produces: Gas,
}

impl Reaction {
    fn new(container: ContainerId, consumes: Gas, produces: Gas) -> Self {
        assert!(consumes.o2 >= 0, "consumes.o2 must be non-negative");
        assert!(consumes.co2 >= 0, "consumes.co2 must be non-negative");
        assert!(consumes.co >= 0, "consumes.co must be non-negative");
        assert!(produces.o2 >= 0, "produces.o2 must be non-negative");
        assert!(produces.co2 >= 0, "produces.co2 must be non-negative");
        assert!(produces.co >= 0, "produces.co must be non-negative");
        Self {
            container,
            consumes,
            produces,
        }
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
        consumes: Gas,
        produces: Gas,
    ) {
        self.reactions
            .push(Reaction::new(container, consumes, produces));
    }

    pub fn tick(&mut self) {
        for reaction in self.reactions.iter().copied() {
            let container = &mut self.containers[reaction.container.index()];
            if !container.gas.can_consume(reaction.consumes) {
                continue;
            }
            container.gas.consume(reaction.consumes);
            container.gas.produce(reaction.produces);
        }
    }

    fn insert_container(&mut self, volume: Volume, gas: Gas) -> ContainerId {
        let id = ContainerId(self.containers.len());
        self.containers.push(Container::new(volume, gas));
        id
    }
}

pub fn add_human(engine: &mut Engine, container: ContainerId, o2_per_tick: i32) {
    engine.add_reaction(
        container,
        Gas {
            o2: o2_per_tick,
            co2: 0,
            co: 0,
        },
        Gas {
            o2: 0,
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
    engine.add_reaction(
        container,
        Gas {
            o2: 0,
            co2: co2_per_tick,
            co: 0,
        },
        Gas {
            o2: o2_per_tick,
            co2: 0,
            co: co_per_tick,
        },
    );
}
