use std::collections::HashSet;

use crate::intu_ir::basicblock::BasicBlock;
use crate::intu_ir::operand::Operand;

use crate::intu_ir::terminator::Terminator;
use crate::intu_ir::{instruction::Instruction, name::Name};

pub struct IntuInstru {
    pub instr: Instruction,
    pub live: HashSet<Name>,
}

pub struct IntuTerm {
    pub term: Terminator,
    pub live: HashSet<Name>,
}

pub struct IntuBasicBlock {
    pub name: Name,
    pub intu_instrs: Vec<IntuInstru>,
    pub intu_term: IntuTerm,
    pub live: HashSet<Name>,
}

fn liveness_analysis(block: &BasicBlock) -> IntuBasicBlock {
    let mut live = HashSet::new();
    let mut intu_instrs = vec![];

    read_terminator(&block.term, &mut live);
    let intu_term = IntuTerm { term: block.term.clone(), live: live.clone() };

    for instr in block.instrs.iter().rev() {
        write_instr(instr, &mut live);
        read_instr(instr, &mut live);
        intu_instrs.push(IntuInstru { instr: instr.clone(), live: live.clone() });
    }
    intu_instrs.reverse();

    IntuBasicBlock { name: block.name.clone(), intu_instrs, intu_term, live }
}

fn insert_operand(operand: &Operand, live: &mut HashSet<Name>) {
    if let Operand::LocalOperand { name, .. } = operand {
        live.insert(name.clone());
    }
}

fn read_terminator(term: &Terminator, live: &mut HashSet<Name>) {
    match term {
        Terminator::Ret { return_operand } => {
            if let Some(operand) = return_operand {
                insert_operand(operand, live);
            }
        },
        Terminator::Br { .. } => {},
        Terminator::CondBr { condition, .. } => {
            insert_operand(condition, live);
        },
        Terminator::IndirectBr { operand, .. } => {
            insert_operand(operand, live);
        },
        Terminator::Unreachable => {},
    }
}

fn read_instr(instr: &Instruction, live: &mut HashSet<Name>) {
    match instr {
        Instruction::Add { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::Sub { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::Mul { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::UDiv { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::SDiv { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::URem { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::SRem { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::And { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::Or { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::Xor { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::Shl { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::LShr { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::AShr { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::FAdd { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::FSub { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::FMul { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::FDiv { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::FRem { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::FNeg { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::ExtractValue { aggregate, .. } => {
            insert_operand(aggregate, live);
        },
        Instruction::InsertValue { aggregate, element, .. } => {
            insert_operand(aggregate, live);
            insert_operand(element, live);
        },
        Instruction::Alloca { num_elements, .. } => {
            insert_operand(num_elements, live);
        },
        Instruction::Load { address, .. } => {
            insert_operand(address, live);
        },
        Instruction::Store { address, value, .. } => {
            insert_operand(address, live);
            insert_operand(value, live);
        },
        Instruction::GetElementPtr { address, indices, .. } => {
            insert_operand(address, live);
            for idx in indices {
                insert_operand(idx, live);
            }
        },
        Instruction::Trunc { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::ZExt { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::SExt { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::FPTrunc { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::FPExt { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::FPToUI { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::FPToSI { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::UIToFP { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::SIToFP { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::PtrToInt { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::IntToPtr { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::BitCast { operand, .. } => {
            insert_operand(operand, live);
        },
        Instruction::ICmp { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::FCmp { operand0, operand1, .. } => {
            insert_operand(operand0, live);
            insert_operand(operand1, live);
        },
        Instruction::Call { function, arguments, .. } => {
            insert_operand(function, live);
            for arg in arguments {
                insert_operand(arg, live);
            }
        },
    }
}
fn write_instr(instr: &Instruction, live: &mut HashSet<Name>) {
    match instr {
        Instruction::Add { dest, .. } => {
            live.remove(dest);
        },
        Instruction::Sub { dest, .. } => {
            live.remove(dest);
        },
        Instruction::Mul { dest, .. } => {
            live.remove(dest);
        },
        Instruction::UDiv { dest, .. } => {
            live.remove(dest);
        },
        Instruction::SDiv { dest, .. } => {
            live.remove(dest);
        },
        Instruction::URem { dest, .. } => {
            live.remove(dest);
        },
        Instruction::SRem { dest, .. } => {
            live.remove(dest);
        },
        Instruction::And { dest, .. } => {
            live.remove(dest);
        },
        Instruction::Or { dest, .. } => {
            live.remove(dest);
        },
        Instruction::Xor { dest, .. } => {
            live.remove(dest);
        },
        Instruction::Shl { dest, .. } => {
            live.remove(dest);
        },
        Instruction::LShr { dest, .. } => {
            live.remove(dest);
        },
        Instruction::AShr { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FAdd { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FSub { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FMul { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FDiv { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FRem { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FNeg { dest, .. } => {
            live.remove(dest);
        },
        Instruction::ExtractValue { dest, .. } => {
            live.remove(dest);
        },
        Instruction::InsertValue { dest, .. } => {
            live.remove(dest);
        },
        Instruction::Alloca { dest, .. } => {
            live.remove(dest);
        },
        Instruction::Load { dest, .. } => {
            live.remove(dest);
        },
        Instruction::Store { .. } => {},
        Instruction::GetElementPtr { dest, .. } => {
            live.remove(dest);
        },
        Instruction::Trunc { dest, .. } => {
            live.remove(dest);
        },
        Instruction::ZExt { dest, .. } => {
            live.remove(dest);
        },
        Instruction::SExt { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FPTrunc { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FPExt { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FPToUI { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FPToSI { dest, .. } => {
            live.remove(dest);
        },
        Instruction::UIToFP { dest, .. } => {
            live.remove(dest);
        },
        Instruction::SIToFP { dest, .. } => {
            live.remove(dest);
        },
        Instruction::PtrToInt { dest, .. } => {
            live.remove(dest);
        },
        Instruction::IntToPtr { dest, .. } => {
            live.remove(dest);
        },
        Instruction::BitCast { dest, .. } => {
            live.remove(dest);
        },
        Instruction::ICmp { dest, .. } => {
            live.remove(dest);
        },
        Instruction::FCmp { dest, .. } => {
            live.remove(dest);
        },
        Instruction::Call { dest, .. } => {
            if let Some(name) = dest {
                live.remove(name);
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intu_ir::constant::{Constant, ConstantRef};
    use crate::intu_ir::types::Types;

    fn mk_local(name: &str) -> Operand {
        Operand::LocalOperand { name: Name::Name(name.into()), ty: Types::new().i32() }
    }

    fn mk_const_int(value: u64) -> Operand {
        Operand::ConstantOperand(ConstantRef::new(Constant::Int { bits: 32, value }))
    }

    fn mk_bb(instrs: Vec<Instruction>, term: Terminator) -> BasicBlock {
        BasicBlock { name: Name::Name("bb".into()), instrs, term }
    }

    fn names(names: &[&str]) -> HashSet<Name> {
        names.iter().map(|s| Name::Name(s.to_string())).collect()
    }

    fn ret(operand: Operand) -> Terminator {
        Terminator::Ret { return_operand: Some(operand) }
    }

    fn add(operand0: Operand, operand1: Operand, dest: &str) -> Instruction {
        Instruction::Add { operand0, operand1, dest: Name::Name(dest.into()) }
    }

    fn analyze(bb: &BasicBlock) -> IntuBasicBlock {
        liveness_analysis(bb)
    }

    #[test]
    fn preserves_instrs_in_order() {
        let a = mk_local("a");
        let bb = mk_bb(
            vec![
                add(mk_const_int(1), mk_const_int(2), "a"),
                add(a, mk_const_int(3), "b"),
            ],
            ret(mk_local("b")),
        );
        let result = analyze(&bb);
        assert_eq!(result.name, bb.name);
        assert_eq!(result.intu_instrs.len(), 2);
        assert_eq!(result.intu_instrs[0].instr, bb.instrs[0]);
        assert_eq!(result.intu_instrs[1].instr, bb.instrs[1]);
        assert_eq!(result.intu_term.term, bb.term);
    }

    #[test]
    fn ret_operand_is_live_in_terminator() {
        let a = mk_local("a");
        let bb = mk_bb(vec![], ret(a));
        let result = analyze(&bb);
        assert_eq!(result.intu_term.live, names(&["a"]));
        assert_eq!(result.live, names(&["a"]));
    }

    #[test]
    fn ret_void_reads_nothing() {
        let bb = mk_bb(vec![], Terminator::Ret { return_operand: None });
        let result = analyze(&bb);
        assert_eq!(result.intu_term.live, HashSet::new());
        assert_eq!(result.live, HashSet::new());
    }

    #[test]
    fn constant_operands_are_not_live() {
        let bb = mk_bb(vec![], ret(mk_const_int(5)));
        let result = analyze(&bb);
        assert_eq!(result.intu_term.live, HashSet::new());
    }

    #[test]
    fn kill_then_gen_per_instruction() {
        let b = mk_local("b");
        let c = mk_local("c");
        let a = mk_local("a");
        let bb = mk_bb(vec![add(b.clone(), c.clone(), "a"), add(a.clone(), mk_const_int(3), "d")], ret(mk_local("d")));
        let result = analyze(&bb);
        assert_eq!(result.intu_term.live, names(&["d"]));
        assert_eq!(result.intu_instrs[1].live, names(&["a"]));
        assert_eq!(result.intu_instrs[0].live, names(&["b", "c"]));
        assert_eq!(result.live, names(&["b", "c"]));
    }

    #[test]
    fn overwritten_definition_is_dead() {
        let a = mk_local("a");
        let bb = mk_bb(
            vec![
                add(mk_const_int(1), mk_const_int(2), "a"),
                add(mk_const_int(3), mk_const_int(4), "a"),
            ],
            ret(a),
        );
        let result = analyze(&bb);
        assert_eq!(result.intu_term.live, names(&["a"]));
        assert_eq!(result.intu_instrs[1].live, HashSet::new());
        assert_eq!(result.intu_instrs[0].live, HashSet::new());
        assert_eq!(result.live, HashSet::new());
    }

    #[test]
    fn unused_definition_is_not_live() {
        let bb = mk_bb(vec![add(mk_const_int(1), mk_const_int(2), "a")], Terminator::Unreachable);
        let result = analyze(&bb);
        assert_eq!(result.intu_term.live, HashSet::new());
        assert_eq!(result.intu_instrs[0].live, HashSet::new());
        assert_eq!(result.live, HashSet::new());
    }

    #[test]
    fn br_does_not_read_dest_block() {
        let bb = mk_bb(vec![], Terminator::Br { dest: Name::Name("next".into()) });
        let result = analyze(&bb);
        assert_eq!(result.intu_term.live, HashSet::new());
        assert_eq!(result.live, HashSet::new());
    }

    #[test]
    fn condbr_reads_condition_only() {
        let cond = mk_local("cond");
        let bb = mk_bb(
            vec![],
            Terminator::CondBr {
                condition: cond,
                true_dest: Name::Name("then".into()),
                false_dest: Name::Name("else".into()),
            },
        );
        let result = analyze(&bb);
        assert_eq!(result.intu_term.live, names(&["cond"]));
        assert_eq!(result.live, names(&["cond"]));
    }

    #[test]
    fn indirectbr_reads_operand_only() {
        let addr = mk_local("addr");
        let bb = mk_bb(
            vec![],
            Terminator::IndirectBr { operand: addr.clone(), possible_dests: vec![Name::Name("l1".into())] },
        );
        let result = analyze(&bb);
        assert_eq!(result.intu_term.live, names(&["addr"]));
    }

    #[test]
    fn store_reads_address_and_value() {
        let p = mk_local("p");
        let v = mk_local("v");
        let bb = mk_bb(
            vec![Instruction::Store { address: p.clone(), value: v.clone(), alignment: 4 }],
            Terminator::Unreachable,
        );
        let result = analyze(&bb);
        assert_eq!(result.intu_instrs[0].live, names(&["p", "v"]));
        assert_eq!(result.live, names(&["p", "v"]));
    }

    #[test]
    fn insertvalue_reads_element() {
        let agg = mk_local("agg");
        let elem = mk_local("elem");
        let bb = mk_bb(
            vec![Instruction::InsertValue {
                aggregate: agg.clone(),
                element: elem.clone(),
                indices: vec![0],
                dest: Name::Name("v".into()),
            }],
            Terminator::Unreachable,
        );
        let result = analyze(&bb);
        assert_eq!(result.intu_instrs[0].live, names(&["agg", "elem"]));
    }

    #[test]
    fn alloca_reads_num_elements() {
        let n = mk_local("n");
        let types = Types::new();
        let bb = mk_bb(
            vec![Instruction::Alloca {
                allocated_type: types.i32(),
                num_elements: n.clone(),
                dest: Name::Name("p".into()),
                alignment: 4,
            }],
            Terminator::Unreachable,
        );
        let result = analyze(&bb);
        assert_eq!(result.intu_instrs[0].live, names(&["n"]));
    }

    #[test]
    fn gep_reads_address_and_indices() {
        let addr = mk_local("addr");
        let idx = mk_local("idx");
        let types = Types::new();
        let bb = mk_bb(
            vec![Instruction::GetElementPtr {
                address: addr.clone(),
                indices: vec![idx.clone()],
                dest: Name::Name("p".into()),
                source_element_type: types.i32(),
            }],
            Terminator::Unreachable,
        );
        let result = analyze(&bb);
        assert_eq!(result.intu_instrs[0].live, names(&["addr", "idx"]));
    }

    #[test]
    fn call_reads_function_and_args() {
        let f = mk_local("f");
        let x = mk_local("x");
        let types = Types::new();
        let bb = mk_bb(
            vec![Instruction::Call {
                function: f.clone(),
                function_ty: types.void(),
                arguments: vec![x.clone()],
                dest: None,
                is_tail_call: false,
            }],
            Terminator::Unreachable,
        );
        let result = analyze(&bb);
        assert_eq!(result.intu_instrs[0].live, names(&["f", "x"]));
    }

    #[test]
    fn call_with_dest_kills_dest() {
        let f = mk_local("f");
        let r = mk_local("r");
        let types = Types::new();
        let bb = mk_bb(
            vec![Instruction::Call {
                function: f.clone(),
                function_ty: types.i32(),
                arguments: vec![],
                dest: Some(Name::Name("r".into())),
                is_tail_call: false,
            }],
            ret(r),
        );
        let result = analyze(&bb);
        assert_eq!(result.intu_term.live, names(&["r"]));
        assert_eq!(result.intu_instrs[0].live, names(&["f"]));
        assert_eq!(result.live, names(&["f"]));
    }
}
