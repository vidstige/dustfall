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
}

impl Gas {
    pub fn new(o2: i32, co2: i32) -> Self {
        Self { o2, co2 }
    }

    pub fn partial_pressure(amount: i32, volume: Volume) -> i32 {
        amount / volume.value()
    }

    pub fn pressure(&self, volume: Volume) -> i32 {
        Self::partial_pressure(self.o2, volume) + Self::partial_pressure(self.co2, volume)
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

#[derive(Debug)]
pub struct Engine {
    containers: Vec<Container>,
    pipes: Vec<Pipe>,
    root: ContainerId,
}

impl Engine {
    pub fn new(volume: Volume, gas: Gas) -> Self {
        let mut engine = Self {
            containers: Vec::new(),
            pipes: Vec::new(),
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

    fn insert_container(&mut self, volume: Volume, gas: Gas) -> ContainerId {
        let id = ContainerId(self.containers.len());
        self.containers.push(Container::new(volume, gas));
        id
    }
}
