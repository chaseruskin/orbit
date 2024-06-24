use super::algo::IpFileNode;

enum Format {
    Tsv,
    Json,
}

impl Default for Format {
    fn default() -> Self {
        Self::Tsv
    }
}

enum Instruction<'a> {
    Hdl(IpFileNode<'a>),
    Supportive(String),
}

struct Blueprint<'a> {
    format: Format,
    steps: Vec<Instruction<'a>>,
}

impl<'a> Default for Blueprint<'a> {
    fn default() -> Self {
        Self {
            format: Format::default(),
            steps: Vec::default(),
        }
    }
}

impl<'a> Blueprint<'a> {
    pub fn new(fmt: Format) -> Self {
        Self {
            format: fmt,
            steps: Vec::new(),
        }
    }

    /// Add the next instruction `instr` to the blueprint.
    pub fn add(&mut self, instr: Instruction<'a>) {
        self.steps.push(instr);
    }
}
