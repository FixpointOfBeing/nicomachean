use std::collections::HashSet;

use crate::riscv::Label;
use crate::riscv_var::{RvVarBasicBlock, RvVarInstr, RvVarLocation};

pub struct RvVarInstrLiveness {
    pub instr: RvVarInstr,
    pub live_before: HashSet<RvVarLocation>,
}

pub struct RvVarBasicBlockLiveness {
    pub name: Label,
    pub instrs: Vec<RvVarInstrLiveness>,
    pub live_in: HashSet<RvVarLocation>,
}

impl RvVarBasicBlockLiveness {
    pub fn all_locations(&self) -> HashSet<RvVarLocation> {
        let mut locations = HashSet::<RvVarLocation>::new();
        for instr_live in &self.instrs {
            let instr = &instr_live.instr;
            let sources = instr.source_locations();
            locations.extend(sources);
            if let Some(dest) = instr.dest_location() {
                locations.insert(dest);
            }
        }
        locations
    }
}

fn read_instr(instr: &RvVarInstr, live_before: &mut HashSet<RvVarLocation>) {
    live_before.extend(instr.source_locations());
}

fn write_instr(instr: &RvVarInstr, live_before: &mut HashSet<RvVarLocation>) {
    if let Some(dest) = instr.dest_location() {
        live_before.remove(&dest);
    }
}

pub fn liveness_analysis(basic_block: &RvVarBasicBlock) -> RvVarBasicBlockLiveness {
    let mut live= HashSet::new();
    let mut instrs = Vec::new();
    for instr in basic_block.instrs.iter().rev() {
        write_instr(instr, &mut live);
        read_instr(instr, &mut live);
        instrs.push(RvVarInstrLiveness { instr: instr.clone(), live_before: live.clone() });
    }
    instrs.reverse();
    RvVarBasicBlockLiveness { name: basic_block.name.clone(), instrs: instrs, live_in: live }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::riscv::rv64imfd_imm::Imm12;
    use crate::riscv::rv64imfd_instr::Rm;
    use crate::riscv_var::location::var;

    fn loc(name: &str) -> RvVarLocation {
        var(name.to_string())
    }

    fn locs(names: &[&str]) -> HashSet<RvVarLocation> {
        names.iter().map(|s| loc(s)).collect()
    }

    fn mk_bb(instrs: Vec<RvVarInstr>) -> RvVarBasicBlock {
        RvVarBasicBlock { name: Label::new("bb".into()), instrs }
    }

    fn add(rd: &str, rs1: &str, rs2: &str) -> RvVarInstr {
        RvVarInstr::Add { rd: loc(rd), rs1: loc(rs1), rs2: loc(rs2) }
    }

    fn analyze(bb: &RvVarBasicBlock) -> RvVarBasicBlockLiveness {
        liveness_analysis(bb)
    }

    #[test]
    fn preserves_name_and_instr_order() {
        let bb = mk_bb(vec![add("a", "b", "c"), add("d", "a", "e")]);
        let result = analyze(&bb);
        assert_eq!(result.name, bb.name);
        assert_eq!(result.instrs.len(), 2);
        assert_eq!(result.instrs[0].instr, bb.instrs[0]);
        assert_eq!(result.instrs[1].instr, bb.instrs[1]);
    }

    #[test]
    fn kill_then_gen_per_instruction() {
        let bb = mk_bb(vec![add("a", "b", "c"), add("d", "a", "e")]);
        let result = analyze(&bb);
        assert_eq!(result.instrs[1].live_before, locs(&["a", "e"]));
        assert_eq!(result.instrs[0].live_before, locs(&["b", "c", "e"]));
        assert_eq!(result.live_in, locs(&["b", "c", "e"]));
    }

    #[test]
    fn overwritten_definition_is_dead() {
        let bb = mk_bb(vec![add("a", "b", "c"), add("a", "d", "e"), add("f", "a", "g")]);
        let result = analyze(&bb);
        assert_eq!(result.instrs[2].live_before, locs(&["a", "g"]));
        assert_eq!(result.instrs[1].live_before, locs(&["d", "e", "g"]));
        assert_eq!(result.instrs[0].live_before, locs(&["b", "c", "d", "e", "g"]));
        assert_eq!(result.live_in, locs(&["b", "c", "d", "e", "g"]));
    }

    #[test]
    fn unused_definition_is_not_live() {
        let bb = mk_bb(vec![add("a", "b", "c")]);
        let result = analyze(&bb);
        assert_eq!(result.instrs[0].live_before, locs(&["b", "c"]));
        assert_eq!(result.live_in, locs(&["b", "c"]));
    }

    #[test]
    fn load_kills_dest_and_reads_address() {
        let bb = mk_bb(vec![RvVarInstr::Lw { rd: loc("a"), rs1: loc("p"), imm: Imm12::from_i16(0) }]);
        let result = analyze(&bb);
        assert_eq!(result.instrs[0].live_before, locs(&["p"]));
        assert_eq!(result.live_in, locs(&["p"]));
    }

    #[test]
    fn store_reads_address_and_value() {
        let bb = mk_bb(vec![RvVarInstr::Sd { rs2: loc("v"), rs1: loc("p"), imm: Imm12::from_i16(0) }]);
        let result = analyze(&bb);
        assert_eq!(result.instrs[0].live_before, locs(&["p", "v"]));
        assert_eq!(result.live_in, locs(&["p", "v"]));
    }

    #[test]
    fn float_store_reads_address_and_value() {
        let bb = mk_bb(vec![RvVarInstr::Fsd { rs2: loc("v"), rs1: loc("p"), imm: Imm12::from_i16(0) }]);
        let result = analyze(&bb);
        assert_eq!(result.instrs[0].live_before, locs(&["p", "v"]));
        assert_eq!(result.live_in, locs(&["p", "v"]));
    }

    #[test]
    fn branch_reads_sources_and_kills_nothing() {
        let bb = mk_bb(vec![RvVarInstr::Beq { rs1: loc("x"), rs2: loc("y"), label: Label::new("next".into()) }]);
        let result = analyze(&bb);
        assert_eq!(result.instrs[0].live_before, locs(&["x", "y"]));
        assert_eq!(result.live_in, locs(&["x", "y"]));
    }

    #[test]
    fn lui_has_no_sources() {
        let bb = mk_bb(vec![RvVarInstr::Lui {
            rd: loc("a"),
            imm: crate::riscv::rv64imfd_imm::Imm32LowZeroBits12::from_i32(0),
        }]);
        let result = analyze(&bb);
        assert_eq!(result.instrs[0].live_before, HashSet::new());
        assert_eq!(result.live_in, HashSet::new());
    }

    #[test]
    fn fadd_reads_rs1_rs2() {
        let bb = mk_bb(vec![RvVarInstr::FaddS { rd: loc("d"), rs1: loc("x"), rs2: loc("y"), rm: Rm::Rne }]);
        let result = analyze(&bb);
        assert_eq!(result.instrs[0].live_before, locs(&["x", "y"]));
        assert_eq!(result.live_in, locs(&["x", "y"]));
    }
}
