use std::fmt;
use crate::riscv::Label;
use crate::riscv::RvInstr;

#[derive(Clone, PartialEq)]
pub struct RvBasicBlock {
    pub name: Label,
    pub instrs: Vec<RvInstr>,
}

impl RvBasicBlock {
    pub fn new(name: Label) -> Self {
        RvBasicBlock { name, instrs: vec![] }
    }
}

impl fmt::Display for RvBasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", self.name)?;
        for instr in &self.instrs {
            writeln!(f, "    {instr}")?;
        }
        Ok(())
    }
}
