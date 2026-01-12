#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContainerId(usize);

impl ContainerId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContainerPath {
    parts: Vec<ContainerId>,
}

impl ContainerPath {
    pub fn new(parts: Vec<ContainerId>) -> Self {
        Self { parts }
    }

    pub fn root(root: ContainerId) -> Self {
        Self { parts: vec![root] }
    }

    pub fn parts(&self) -> &[ContainerId] {
        &self.parts
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
pub struct Gas {
    pub o: i32,
    pub co2: i32,
}

impl Gas {
    pub fn new(o: i32, co2: i32) -> Self {
        Self { o, co2 }
    }
}

#[derive(Debug, Clone)]
pub struct Container {
    volume: Volume,
    gas: Gas,
    parent: Option<ContainerId>,
    children: Vec<ContainerId>,
}

impl Container {
    fn new(volume: Volume, gas: Gas, parent: Option<ContainerId>) -> Self {
        Self {
            volume,
            gas,
            parent,
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

    pub fn parent(&self) -> Option<ContainerId> {
        self.parent
    }

    pub fn children(&self) -> &[ContainerId] {
        &self.children
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipe {
    pub a: ContainerPath,
    pub b: ContainerPath,
}

impl Pipe {
    pub fn new(a: ContainerPath, b: ContainerPath) -> Self {
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
        let id = engine.insert_container(volume, gas, None);
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
        let id = self.insert_container(volume, gas, Some(parent));
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

    pub fn add_pipe(&mut self, a: ContainerPath, b: ContainerPath) {
        self.pipes.push(Pipe::new(a, b));
    }

    pub fn add_pipe_between(&mut self, a: ContainerId, b: ContainerId) {
        let a_path = self.path_to(a);
        let b_path = self.path_to(b);
        self.add_pipe(a_path, b_path);
    }

    pub fn path_to(&self, id: ContainerId) -> ContainerPath {
        let mut parts = Vec::new();
        let mut current = Some(id);
        while let Some(container_id) = current {
            parts.push(container_id);
            current = self.containers[container_id.index()].parent;
        }
        parts.reverse();
        ContainerPath::new(parts)
    }

    pub fn resolve_path(&self, path: &ContainerPath) -> Option<ContainerId> {
        let mut iter = path.parts().iter().copied();
        let root = iter.next()?;
        if root != self.root {
            return None;
        }
        let mut current = root;
        for next in iter {
            let container = &self.containers[current.index()];
            if !container.children.contains(&next) {
                return None;
            }
            current = next;
        }
        Some(current)
    }

    fn insert_container(
        &mut self,
        volume: Volume,
        gas: Gas,
        parent: Option<ContainerId>,
    ) -> ContainerId {
        let id = ContainerId(self.containers.len());
        self.containers.push(Container::new(volume, gas, parent));
        id
    }
}
