use crate::riscv::RvBasicBlock;
use std::fmt;

pub struct RvProgram {
    pub blocks: Vec<RvBasicBlock>,
}

impl RvProgram {
    pub fn new() -> Self {
        let blocks = vec![];
        RvProgram { blocks }
    }

    pub fn append_basic_block(&mut self, block: RvBasicBlock) {
        self.blocks.push(block);
    }
}

impl fmt::Display for RvProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, block) in self.blocks.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{block}")?;
        }
        Ok(())
    }
}

