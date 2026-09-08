use crate::{
    explicate_control::{CAtom, CExpr, CStmt, CTail},
    gensym::Gensym,
    riscv::rv64imfd_instr::Rm,
    riscv_var::{
        basicblock::RvVarBasicBlock,
        instruction::{RvVarInstr, fmv_d, fneg_d, li, mv, seqz, snez},
        location::{RvVarLocation, a0, fa0, ft0, ft1, ra, t0, t1, x0, zero},
        program::RvVarProgram,
    },
    syntax::{BinOp, Type, UnaryOp},
};

use crate::riscv::Label;
use crate::riscv::Imm12;

fn bool_not(rd: RvVarLocation, rs1: RvVarLocation) -> RvVarInstr {
    RvVarInstr::Xori { rd, rs1: rs1, imm: Imm12::from_i16(1) }
}

fn select_atom(atom: CAtom, instrs: &mut Vec<RvVarInstr>, dest: RvVarLocation) {
    match atom {
        CAtom::Unit => {
            let instr = RvVarInstr::Addi { rd: dest, rs1: zero(), imm: Imm12::from_i16(0) };
            instrs.push(instr);
        },
        CAtom::Bool(b) => {
            let v = if b { 1 } else { 0 };
            let instr = RvVarInstr::Addi { rd: dest, rs1: zero(), imm: Imm12::from_i16(v as i16) };
            instrs.push(instr);
        },
        CAtom::Int(i) => instrs.append(&mut li(dest, i)),
        CAtom::Float(f) => {
            let bits = f.to_bits() as i64;
            instrs.append(&mut li(t0(), bits));
            instrs.push(RvVarInstr::FmvDX { rd: dest, rs1: t0() });
        },
        CAtom::Var(name, ty) => {
            if matches!(ty, Type::Float) {
                let var = RvVarLocation::FVar(name);
                let mv = fmv_d(dest, var);
                instrs.push(mv);
            } else {
                let var = RvVarLocation::XVar(name);
                let mv = mv(dest, var);
                instrs.push(mv);
            }
        },
    };
}

fn is_catom_float_type(atom: &CAtom) -> bool {
    match atom {
        CAtom::Unit => false,
        CAtom::Bool(_) => false,
        CAtom::Int(_) => false,
        CAtom::Float(_) => true,
        CAtom::Var(_, ty) => matches!(ty, Type::Float),
    }
}

fn is_cexpr_float_type(expr: &CExpr) -> bool {
    match expr {
        CExpr::Atom(catom) => is_catom_float_type(catom),
        CExpr::BinOp(op, catom, _) => match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => is_catom_float_type(catom),
            _ => false,
        },
        CExpr::UnaryOp(op, catom) => match op {
            UnaryOp::Neg => is_catom_float_type(catom),
            UnaryOp::Not => false,
        },
        CExpr::Call(_catom, _catoms) => todo!(),
        CExpr::MakeClosure(_catom, _catoms, _) => todo!(),
        CExpr::Project(_catom, _, _) => todo!(),
    }
}

fn select_expr_binop_xreg(op: BinOp, left: CAtom, right: CAtom, instrs: &mut Vec<RvVarInstr>, dest: RvVarLocation) {
    debug_assert!(matches!(dest, RvVarLocation::XVar(_) | RvVarLocation::XReg(_)));

    match op {
        BinOp::Add => {
            select_atom(left, instrs, t0());
            select_atom(right, instrs, t1());
            let instr = RvVarInstr::Add { rd: dest.clone(), rs1: t0(), rs2: t1() };
            instrs.push(instr);
        },
        BinOp::Sub => {
            select_atom(left, instrs, t0());
            select_atom(right, instrs, t1());
            let instr = RvVarInstr::Sub { rd: dest, rs1: t0(), rs2: t1() };
            instrs.push(instr);
        },
        BinOp::Mul => {
            // 乘法溢出的处理（回绕）
            select_atom(left, instrs, t0());
            select_atom(right, instrs, t1());
            let instr = RvVarInstr::Mul { rd: dest, rs1: t0(), rs2: t1() };
            instrs.push(instr);
        },
        BinOp::Div => {
            select_atom(left, instrs, t0());
            select_atom(right, instrs, t1());
            let instr = RvVarInstr::Div { rd: dest, rs1: t0(), rs2: t1() };
            instrs.push(instr);
        },
        BinOp::Eq => {
            select_atom(left, instrs, t0());
            select_atom(right, instrs, t1());
            let sub = RvVarInstr::Sub { rd: dest.clone(), rs1: t0(), rs2: t1() };
            let set_bool = seqz(dest.clone(), dest);
            instrs.push(sub);
            instrs.push(set_bool);
        },
        BinOp::Neq => {
            select_atom(left, instrs, t0());
            select_atom(right, instrs, t1());
            let sub = RvVarInstr::Sub { rd: dest.clone(), rs1: t0(), rs2: t1() };
            let set_bool = snez(dest.clone(), dest);
            instrs.push(sub);
            instrs.push(set_bool);
        },
        BinOp::Lt => {
            select_atom(left, instrs, t0());
            select_atom(right, instrs, t1());
            let instr = RvVarInstr::Slt { rd: dest, rs1: t0(), rs2: t1() };
            instrs.push(instr);
        },
        BinOp::Gt => {
            select_atom(left, instrs, t0());
            select_atom(right, instrs, t1());
            let instr = RvVarInstr::Slt { rd: dest, rs1: t1(), rs2: t0() };
            instrs.push(instr);
        },
        BinOp::Leq => {
            // left <= right => !(left > right)
            select_atom(left, instrs, t0());
            select_atom(right, instrs, t1());
            let right_lt_left = RvVarInstr::Slt { rd: dest.clone(), rs1: t1(), rs2: t0() };
            let reverse = bool_not(dest.clone(), dest);
            instrs.push(right_lt_left);
            instrs.push(reverse);
        },
        BinOp::Geq => {
            // left >= right => !(left < right)
            select_atom(left, instrs, t0());
            select_atom(right, instrs, t1());
            let left_lt_right = RvVarInstr::Slt { rd: dest.clone(), rs1: t0(), rs2: t1() };

            let reverse = bool_not(dest.clone(), dest);
            instrs.push(left_lt_right);
            instrs.push(reverse);
        },
        _ => unreachable!(),
    }
}

fn select_expr_binop_freg(op: BinOp, left: CAtom, right: CAtom, instrs: &mut Vec<RvVarInstr>, dest: RvVarLocation) {
    match op {
        BinOp::Add => {
            debug_assert!(matches!(dest, RvVarLocation::FVar(_) | RvVarLocation::FReg(_)));

            select_atom(left, instrs, ft0());
            select_atom(right, instrs, ft1());
            let instr = RvVarInstr::FaddD { rd: dest, rs1: ft0(), rs2: ft1(), rm: Rm::Rne };
            instrs.push(instr);
        },
        BinOp::Sub => {
            debug_assert!(matches!(dest, RvVarLocation::FVar(_) | RvVarLocation::FReg(_)));

            select_atom(left, instrs, ft0());
            select_atom(right, instrs, ft1());
            let instr = RvVarInstr::FsubD { rd: dest, rs1: ft0(), rs2: ft1(), rm: Rm::Rne };
            instrs.push(instr);
        },
        BinOp::Mul => {
            debug_assert!(matches!(dest, RvVarLocation::FVar(_) | RvVarLocation::FReg(_)));

            select_atom(left, instrs, ft0());
            select_atom(right, instrs, ft1());
            let instr = RvVarInstr::FmulD { rd: dest, rs1: ft0(), rs2: ft1(), rm: Rm::Rne };
            instrs.push(instr);
        },
        BinOp::Div => {
            debug_assert!(matches!(dest, RvVarLocation::FVar(_) | RvVarLocation::FReg(_)));

            select_atom(left, instrs, ft0());
            select_atom(right, instrs, ft1());
            let instr = RvVarInstr::FdivD { rd: dest, rs1: ft0(), rs2: ft1(), rm: Rm::Rne };
            instrs.push(instr);
        },
        BinOp::Eq => {
            debug_assert!(matches!(dest, RvVarLocation::XVar(_) | RvVarLocation::XReg(_)));

            select_atom(left, instrs, ft0());
            select_atom(right, instrs, ft1());
            let instr = RvVarInstr::FeqD { rd: dest, rs1: ft0(), rs2: ft1() };
            instrs.push(instr);
        },
        BinOp::Neq => {
            debug_assert!(matches!(dest, RvVarLocation::XVar(_) | RvVarLocation::XReg(_)));

            select_atom(left, instrs, ft0());
            select_atom(right, instrs, ft1());
            let eq = RvVarInstr::FeqD { rd: dest.clone(), rs1: ft0(), rs2: ft1() };
            let reverse = bool_not(dest.clone(), dest);
            instrs.push(eq);
            instrs.push(reverse);
        },
        BinOp::Lt => {
            debug_assert!(matches!(dest, RvVarLocation::XVar(_) | RvVarLocation::XReg(_)));

            select_atom(left, instrs, ft0());
            select_atom(right, instrs, ft1());
            let instr = RvVarInstr::FltD { rd: dest, rs1: ft0(), rs2: ft1() };
            instrs.push(instr);
        },
        BinOp::Gt => {
            debug_assert!(matches!(dest, RvVarLocation::XVar(_) | RvVarLocation::XReg(_)));

            // left > right => right < left
            select_atom(left, instrs, ft0());
            select_atom(right, instrs, ft1());
            let instr = RvVarInstr::FltD { rd: dest, rs1: ft1(), rs2: ft0() };
            instrs.push(instr);
        },
        BinOp::Leq => {
            debug_assert!(matches!(dest, RvVarLocation::XVar(_) | RvVarLocation::XReg(_)));

            select_atom(left, instrs, ft0());
            select_atom(right, instrs, ft1());
            let instr = RvVarInstr::FleD { rd: dest, rs1: ft0(), rs2: ft1() };
            instrs.push(instr);
        },
        BinOp::Geq => {
            debug_assert!(matches!(dest, RvVarLocation::XVar(_) | RvVarLocation::XReg(_)));

            // left >= right => right <= left
            select_atom(left, instrs, ft0());
            select_atom(right, instrs, ft1());
            let instr = RvVarInstr::FleD { rd: dest, rs1: ft1(), rs2: ft0() };
            instrs.push(instr);
        },
        _ => unreachable!(),
    }
}

fn select_expr_unaryop_freg(op: UnaryOp, atom: CAtom, instrs: &mut Vec<RvVarInstr>, dest: RvVarLocation) {
    match op {
        UnaryOp::Neg => {
            debug_assert!(matches!(dest, RvVarLocation::FVar(_) | RvVarLocation::FReg(_)));

            select_atom(atom, instrs, dest.clone());
            let instr = fneg_d(dest.clone(), dest);
            instrs.push(instr);
        },
        UnaryOp::Not => unreachable!(),
    }
}

fn select_expr_unaryop_xreg(op: UnaryOp, atom: CAtom, instrs: &mut Vec<RvVarInstr>, dest: RvVarLocation) {
    match op {
        UnaryOp::Neg => {
            debug_assert!(matches!(dest, RvVarLocation::XVar(_) | RvVarLocation::XReg(_)));

            select_atom(atom, instrs, dest.clone());
            let sub = RvVarInstr::Sub { rd: dest.clone(), rs1: zero(), rs2: dest };
            instrs.push(sub);
        },
        UnaryOp::Not => unreachable!(),
    }
}

fn select_expr(expr: CExpr, instrs: &mut Vec<RvVarInstr>, dest: RvVarLocation) {
    match expr {
        CExpr::Atom(catom) => {
            select_atom(catom, instrs, dest);
        },
        CExpr::BinOp(op, left, right) => match op {
            BinOp::And => {
                select_atom(left, instrs, t0());
                select_atom(right, instrs, t1());
                let instr = RvVarInstr::And { rd: dest, rs1: t0(), rs2: t1() };
                instrs.push(instr);
            },
            BinOp::Or => {
                select_atom(left, instrs, t0());
                select_atom(right, instrs, t1());
                let instr = RvVarInstr::Or { rd: dest, rs1: t0(), rs2: t1() };
                instrs.push(instr);
            },
            _ => {
                if is_catom_float_type(&left) {
                    select_expr_binop_freg(op, left, right, instrs, dest);
                } else {
                    select_expr_binop_xreg(op, left, right, instrs, dest);
                }
            },
        },
        CExpr::UnaryOp(op, catom) => match op {
            UnaryOp::Neg => {
                if is_catom_float_type(&catom) {
                    select_expr_unaryop_freg(op, catom, instrs, dest);
                } else {
                    select_expr_unaryop_xreg(op, catom, instrs, dest);
                }
            },
            UnaryOp::Not => {
                select_atom(catom, instrs, dest.clone());
                let reverse = bool_not(dest.clone(), dest);
                instrs.push(reverse);
            },
        },
        CExpr::Call(_func, _args) => todo!(),
        CExpr::MakeClosure(_catom, _catoms, _) => todo!(),
        CExpr::Project(_catom, _, _) => todo!(),
    }
}

fn select_stmt(stmt: CStmt, instrs: &mut Vec<RvVarInstr>) {
    match stmt {
        CStmt::Assign(name, cexpr, ty) => {
            let var = if matches!(ty, Type::Float) { RvVarLocation::FVar(name) } else { RvVarLocation::XVar(name) };
            select_expr(cexpr, instrs, var);
        },
    }
}

pub fn select_tail(tail: CTail, gensym: &mut Gensym, mut current: RvVarBasicBlock, prog: &mut RvVarProgram) {
    let instrs = &mut current.instrs;
    match tail {
        CTail::Return(cexpr) => {
            let dest = if is_cexpr_float_type(&cexpr) { fa0() } else { a0() };
            select_expr(cexpr, instrs, dest);
            let instr = RvVarInstr::Jalr { rd: x0(), rs1: ra(), imm: Imm12::from_i16(0) };
            instrs.push(instr);
            prog.append_basic_block(current);
        },
        CTail::TailCall(_func, _args) => {
            // select_atom(func, instrs);
            // for arg in args {
            //     select_atom(arg, instrs);
            // }
            todo!()
        },
        CTail::Seq(cstmt, ctail) => {
            select_stmt(cstmt, instrs);
            select_tail(*ctail, gensym, current, prog)
        },
        CTail::If(catom, then_tail, else_tail) => {
            select_atom(catom, instrs, t0());

            let then_name = gensym.fresh_with_prefix(".Lthen");
            let else_name = gensym.fresh_with_prefix(".Lelse");
            let then_label = Label::new(then_name);
            let else_label = Label::new(else_name);

            let branch = RvVarInstr::Beq { rs1: t0(), rs2: zero(), label: else_label.clone() };
            instrs.push(branch);

            prog.append_basic_block(current);

            let then_block = RvVarBasicBlock::new(then_label);
            select_tail(*then_tail, gensym, then_block, prog);

            let else_block = RvVarBasicBlock::new(else_label);
            select_tail(*else_tail, gensym, else_block, prog);
        },
    }
}

// todo
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum Val {
        Int(i64),
        Float(f64),
    }

    fn sext(v: i64, bits: u32) -> i64 {
        (v << (64 - bits)) >> (64 - bits)
    }

    fn lookup(env: &[(RvVarLocation, Val)], loc: &RvVarLocation) -> Val {
        env.iter().rev().find(|(l, _)| l == loc).unwrap().1.clone()
    }

    fn lookup_int(env: &[(RvVarLocation, Val)], loc: &RvVarLocation) -> i64 {
        match lookup(env, loc) {
            Val::Int(i) => i,
            Val::Float(_) => {
                panic!("expected integer value at {loc}")
            },
        }
    }

    fn lookup_float(env: &[(RvVarLocation, Val)], loc: &RvVarLocation) -> f64 {
        match lookup(env, loc) {
            Val::Float(f) => f,
            Val::Int(_) => panic!("expected float value at {loc}"),
        }
    }

    fn store(env: &mut Vec<(RvVarLocation, Val)>, loc: &RvVarLocation, v: Val) {
        if let Some(pair) = env.iter_mut().find(|(l, _)| l == loc) {
            pair.1 = v;
        } else {
            env.push((loc.clone(), v));
        }
    }

    fn exec_instr(instr: &RvVarInstr, env: &mut Vec<(RvVarLocation, Val)>) {
        match instr {
            RvVarInstr::Addi { rd, rs1, imm } => {
                let v = lookup_int(env, rs1).wrapping_add(imm.to_i16() as i64);
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::Addiw { rd, rs1, imm } => {
                let v = lookup_int(env, rs1).wrapping_add(imm.to_i16() as i64);
                store(env, rd, Val::Int(sext(v, 32)));
            },
            RvVarInstr::Slli { rd, rs1, shamt } => {
                let v = lookup_int(env, rs1) << shamt.to_u8();
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::Lui { rd, imm } => {
                let v = sext((imm.to_i32() >> 12) as i64, 20) << 12;
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::Add { rd, rs1, rs2 } => {
                let v = lookup_int(env, rs1).wrapping_add(lookup_int(env, rs2));
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::Sub { rd, rs1, rs2 } => {
                let v = lookup_int(env, rs1).wrapping_sub(lookup_int(env, rs2));
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::Mul { rd, rs1, rs2 } => {
                let v = lookup_int(env, rs1).wrapping_mul(lookup_int(env, rs2));
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::Div { rd, rs1, rs2 } => {
                let a = lookup_int(env, rs1);
                let b = lookup_int(env, rs2);
                let v = if b == 0 { -1 } else { a / b };
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::Slt { rd, rs1, rs2 } => {
                let v = (lookup_int(env, rs1) < lookup_int(env, rs2)) as i64;
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::Sltu { rd, rs1, rs2 } => {
                let a = lookup_int(env, rs1) as u64;
                let b = lookup_int(env, rs2) as u64;
                store(env, rd, Val::Int((a < b) as i64));
            },
            RvVarInstr::Sltiu { rd, rs1, imm } => {
                let a = lookup_int(env, rs1) as u64;
                let b = imm.to_i16() as u64;
                store(env, rd, Val::Int((a < b) as i64));
            },
            RvVarInstr::Xori { rd, rs1, imm } => {
                let v = lookup_int(env, rs1) ^ (imm.to_i16() as i64);
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::And { rd, rs1, rs2 } => {
                let v = lookup_int(env, rs1) & lookup_int(env, rs2);
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::Or { rd, rs1, rs2 } => {
                let v = lookup_int(env, rs1) | lookup_int(env, rs2);
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::FaddD { rd, rs1, rs2, .. } => {
                let v = lookup_float(env, rs1) + lookup_float(env, rs2);
                store(env, rd, Val::Float(v));
            },
            RvVarInstr::FsubD { rd, rs1, rs2, .. } => {
                let v = lookup_float(env, rs1) - lookup_float(env, rs2);
                store(env, rd, Val::Float(v));
            },
            RvVarInstr::FmulD { rd, rs1, rs2, .. } => {
                let v = lookup_float(env, rs1) * lookup_float(env, rs2);
                store(env, rd, Val::Float(v));
            },
            RvVarInstr::FdivD { rd, rs1, rs2, .. } => {
                let v = lookup_float(env, rs1) / lookup_float(env, rs2);
                store(env, rd, Val::Float(v));
            },
            RvVarInstr::FeqD { rd, rs1, rs2 } => {
                let v = (lookup_float(env, rs1) == lookup_float(env, rs2)) as i64;
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::FltD { rd, rs1, rs2 } => {
                let v = (lookup_float(env, rs1) < lookup_float(env, rs2)) as i64;
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::FleD { rd, rs1, rs2 } => {
                let v = (lookup_float(env, rs1) <= lookup_float(env, rs2)) as i64;
                store(env, rd, Val::Int(v));
            },
            RvVarInstr::FmvDX { rd, rs1 } => {
                let bits = lookup_int(env, rs1) as u64;
                store(env, rd, Val::Float(f64::from_bits(bits)));
            },
            RvVarInstr::FsgnjD { rd, rs1, rs2 } => {
                let a = lookup_float(env, rs1);
                let b = lookup_float(env, rs2);
                store(env, rd, Val::Float(a.abs().copysign(b)));
            },
            RvVarInstr::FsgnjnD { rd, rs1, rs2 } => {
                let a = lookup_float(env, rs1);
                let b = lookup_float(env, rs2);
                store(env, rd, Val::Float(a.abs().copysign(-b)));
            },
            _ => panic!("unexpected instruction in interpreter: {instr:?}"),
        }
    }

    fn read_result(env: &[(RvVarLocation, Val)]) -> Val {
        if env.iter().rev().any(|(l, _)| l == &fa0()) { lookup(env, &fa0()) } else { lookup(env, &a0()) }
    }

    fn interpret(prog: &RvVarProgram) -> Val {
        let mut env: Vec<(RvVarLocation, Val)> = vec![(zero(), Val::Int(0))];
        let mut pc = 0usize;
        loop {
            let block = &prog.blocks[pc];
            let mut jumped = false;
            for instr in &block.instrs {
                match instr {
                    RvVarInstr::Jalr { .. } => {
                        return read_result(&env);
                    },
                    RvVarInstr::Beq { rs1, rs2, label } => {
                        let taken = lookup_int(&env, rs1) == lookup_int(&env, rs2);
                        pc = if taken {
                            prog.blocks.iter().position(|b| b.name.name == label.name).unwrap()
                        } else {
                            pc + 1
                        };
                        jumped = true;
                        break;
                    },
                    _ => exec_instr(instr, &mut env),
                }
            }
            if !jumped {
                pc += 1;
            }
        }
    }

    fn run_tail(tail: CTail) -> Val {
        let mut gensym = Gensym::new();
        let entry = RvVarBasicBlock::new(Label::new("entry".to_string()));
        let mut prog = RvVarProgram::new();
        select_tail(tail, &mut gensym, entry, &mut prog);
        interpret(&prog)
    }

    fn c_int(i: i64) -> CAtom {
        CAtom::Int(i)
    }

    fn c_float(f: f64) -> CAtom {
        CAtom::Float(f)
    }

    fn c_bool(b: bool) -> CAtom {
        CAtom::Bool(b)
    }

    fn c_var(name: &str, ty: Type) -> CAtom {
        CAtom::Var(name.to_string(), ty)
    }

    fn c_atom(a: CAtom) -> CExpr {
        CExpr::Atom(a)
    }

    fn c_binop(op: BinOp, l: CAtom, r: CAtom) -> CExpr {
        CExpr::BinOp(op, l, r)
    }

    fn c_ret(e: CExpr) -> CTail {
        CTail::Return(e)
    }

    fn c_if(c: CAtom, t: CTail, e: CTail) -> CTail {
        CTail::If(c, Box::new(t), Box::new(e))
    }

    fn c_seq(stmt: CStmt, tail: CTail) -> CTail {
        CTail::Seq(stmt, Box::new(tail))
    }

    fn run_expr_int(e: CExpr) -> i64 {
        match run_tail(c_ret(e)) {
            Val::Int(i) => i,
            Val::Float(_) => panic!("expected integer result"),
        }
    }

    fn run_expr_float(e: CExpr) -> f64 {
        match run_tail(c_ret(e)) {
            Val::Float(f) => f,
            Val::Int(_) => panic!("expected float result"),
        }
    }

    // --- 整数加减乘除 ---

    #[test]
    fn int_add() {
        assert_eq!(run_expr_int(c_binop(BinOp::Add, c_int(1), c_int(2))), 3);
    }

    #[test]
    fn int_sub() {
        assert_eq!(run_expr_int(c_binop(BinOp::Sub, c_int(10), c_int(3))), 7);
    }

    #[test]
    fn int_mul() {
        assert_eq!(run_expr_int(c_binop(BinOp::Mul, c_int(6), c_int(7))), 42);
    }

    #[test]
    fn int_div() {
        assert_eq!(run_expr_int(c_binop(BinOp::Div, c_int(8), c_int(2))), 4);
    }

    #[test]
    fn int_div_truncates_toward_zero() {
        assert_eq!(run_expr_int(c_binop(BinOp::Div, c_int(7), c_int(2))), 3);
        assert_eq!(run_expr_int(c_binop(BinOp::Div, c_int(-7), c_int(2))), -3);
    }

    #[test]
    fn int_unary_neg() {
        assert_eq!(run_expr_int(CExpr::UnaryOp(UnaryOp::Neg, c_int(5))), -5);
    }

    // --- 浮点加减乘除 ---

    #[test]
    fn float_add() {
        assert_eq!(run_expr_float(c_binop(BinOp::Add, c_float(1.5), c_float(2.5))), 4.0);
    }

    #[test]
    fn float_sub() {
        assert_eq!(run_expr_float(c_binop(BinOp::Sub, c_float(5.0), c_float(1.5))), 3.5);
    }

    #[test]
    fn float_mul() {
        assert_eq!(run_expr_float(c_binop(BinOp::Mul, c_float(2.0), c_float(3.0))), 6.0);
    }

    #[test]
    fn float_div() {
        assert_eq!(run_expr_float(c_binop(BinOp::Div, c_float(6.0), c_float(2.0))), 3.0);
    }

    #[test]
    fn float_unary_neg() {
        assert_eq!(run_expr_float(CExpr::UnaryOp(UnaryOp::Neg, c_float(1.5))), -1.5);
    }

    // --- 整数比较 ---

    #[test]
    fn int_lt() {
        assert_eq!(run_expr_int(c_binop(BinOp::Lt, c_int(1), c_int(2))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Lt, c_int(2), c_int(1))), 0);
    }

    #[test]
    fn int_gt() {
        assert_eq!(run_expr_int(c_binop(BinOp::Gt, c_int(2), c_int(1))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Gt, c_int(1), c_int(2))), 0);
    }

    #[test]
    fn int_leq() {
        assert_eq!(run_expr_int(c_binop(BinOp::Leq, c_int(1), c_int(1))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Leq, c_int(2), c_int(1))), 0);
    }

    #[test]
    fn int_geq() {
        assert_eq!(run_expr_int(c_binop(BinOp::Geq, c_int(2), c_int(2))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Geq, c_int(1), c_int(2))), 0);
    }

    #[test]
    fn int_eq() {
        assert_eq!(run_expr_int(c_binop(BinOp::Eq, c_int(1), c_int(1))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Eq, c_int(1), c_int(2))), 0);
    }

    #[test]
    fn int_neq() {
        assert_eq!(run_expr_int(c_binop(BinOp::Neq, c_int(1), c_int(2))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Neq, c_int(1), c_int(1))), 0);
    }

    // --- 浮点比较 ---

    #[test]
    fn float_lt() {
        assert_eq!(run_expr_int(c_binop(BinOp::Lt, c_float(1.0), c_float(2.0))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Lt, c_float(2.0), c_float(1.0))), 0);
    }

    #[test]
    fn float_gt() {
        assert_eq!(run_expr_int(c_binop(BinOp::Gt, c_float(2.0), c_float(1.0))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Gt, c_float(1.0), c_float(2.0))), 0);
    }

    #[test]
    fn float_leq() {
        assert_eq!(run_expr_int(c_binop(BinOp::Leq, c_float(1.0), c_float(1.0))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Leq, c_float(2.0), c_float(1.0))), 0);
    }

    #[test]
    fn float_geq() {
        assert_eq!(run_expr_int(c_binop(BinOp::Geq, c_float(2.0), c_float(2.0))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Geq, c_float(1.0), c_float(2.0))), 0);
    }

    #[test]
    fn float_eq() {
        assert_eq!(run_expr_int(c_binop(BinOp::Eq, c_float(1.0), c_float(1.0))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Eq, c_float(1.0), c_float(2.0))), 0);
    }

    // --- 逻辑运算 ---

    #[test]
    fn logic_and() {
        assert_eq!(run_expr_int(c_binop(BinOp::And, c_bool(true), c_bool(false))), 0);
        assert_eq!(run_expr_int(c_binop(BinOp::And, c_bool(true), c_bool(true))), 1);
    }

    #[test]
    fn logic_or() {
        assert_eq!(run_expr_int(c_binop(BinOp::Or, c_bool(false), c_bool(true))), 1);
        assert_eq!(run_expr_int(c_binop(BinOp::Or, c_bool(false), c_bool(false))), 0);
    }

    #[test]
    fn logic_not() {
        assert_eq!(run_expr_int(CExpr::UnaryOp(UnaryOp::Not, c_bool(true))), 0);
        assert_eq!(run_expr_int(CExpr::UnaryOp(UnaryOp::Not, c_bool(false))), 1);
    }

    // --- if ---

    #[test]
    fn if_true_branch() {
        let tail = c_if(c_bool(true), c_ret(c_atom(c_int(1))), c_ret(c_atom(c_int(2))));
        assert_eq!(run_tail(tail), Val::Int(1));
    }

    #[test]
    fn if_false_branch() {
        let tail = c_if(c_bool(false), c_ret(c_atom(c_int(1))), c_ret(c_atom(c_int(2))));
        assert_eq!(run_tail(tail), Val::Int(2));
    }

    #[test]
    fn if_float_result() {
        let tail = c_if(c_bool(true), c_ret(c_atom(c_float(1.5))), c_ret(c_atom(c_float(2.5))));
        assert_eq!(run_tail(tail), Val::Float(1.5));
    }

    // x > 0 ? x : -x  （等价于绝对值）
    fn abs_via_if(x: i64) -> Val {
        let tail = c_seq(
            CStmt::Assign("x".to_string(), c_atom(c_int(x)), Type::Int),
            c_seq(
                CStmt::Assign("c".to_string(), c_binop(BinOp::Gt, c_var("x", Type::Int), c_int(0)), Type::Bool),
                c_if(
                    c_var("c", Type::Bool),
                    c_ret(c_atom(c_var("x", Type::Int))),
                    c_ret(CExpr::UnaryOp(UnaryOp::Neg, c_var("x", Type::Int))),
                ),
            ),
        );
        run_tail(tail)
    }

    #[test]
    fn if_computed_condition() {
        assert_eq!(abs_via_if(5), Val::Int(5));
        assert_eq!(abs_via_if(-5), Val::Int(5));
        assert_eq!(abs_via_if(0), Val::Int(0));
    }

    // --- return ---

    #[test]
    fn return_int() {
        assert_eq!(run_tail(c_ret(c_atom(c_int(42)))), Val::Int(42));
    }

    #[test]
    fn return_bool() {
        assert_eq!(run_tail(c_ret(c_atom(c_bool(true)))), Val::Int(1));
    }

    #[test]
    fn return_float() {
        assert_eq!(run_tail(c_ret(c_atom(c_float(3.14)))), Val::Float(3.14));
    }

    #[test]
    fn return_composite_expr() {
        // (1 + 2) * 3 = 9
        let tail = c_seq(
            CStmt::Assign("a".to_string(), c_binop(BinOp::Add, c_int(1), c_int(2)), Type::Int),
            c_seq(
                CStmt::Assign("b".to_string(), c_binop(BinOp::Mul, c_var("a", Type::Int), c_int(3)), Type::Int),
                c_ret(c_atom(c_var("b", Type::Int))),
            ),
        );
        assert_eq!(run_tail(tail), Val::Int(9));
    }
}
