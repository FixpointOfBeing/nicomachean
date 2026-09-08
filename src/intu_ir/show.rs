use crate::intu_ir::types::Types;

pub trait Show {
    fn show(&self, types: &Types) -> String;
}

mod module_show {
    use crate::intu_ir::module;
    use crate::intu_ir::types::Types;
    use std::fmt::Write;

    use super::Show;

    impl Show for module::Module {
        fn show(&self, types: &Types) -> String {
            let mut parts: Vec<String> = Vec::new();

            let header = format!("source_filename = \"{}\"", self.source_file_name);

            parts.push(header);

            let struct_names: Vec<String> = self.types.all_struct_names().cloned().collect();
            for name in &struct_names {
                if let Some(def) = self.types.named_struct_def(name) {
                    parts.push(format!("%{} = {}", name, def.show(types)));
                }
            }

            for gv in &self.global_vars {
                parts.push(gv.show(types));
            }

            for fd in &self.func_declarations {
                parts.push(fd.show(types));
            }
            for func in &self.functions {
                parts.push(func.show(types));
            }

            parts.join("\n\n") + "\n"
        }
    }

    impl Show for module::GlobalVariable {
        fn show(&self, types: &Types) -> String {
            let mut s = String::new();
            write!(s, "@{} = ", self.name.show(types)).unwrap();

            if self.addr_space != 0 {
                write!(s, "addrspace({}) ", self.addr_space).unwrap();
            }
            write!(s, "{} ", if self.is_constant { "constant" } else { "global" }).unwrap();
            write!(s, "{}", self.ty.show(types)).unwrap();
            if let Some(ref init) = self.initializer {
                write!(s, " {}", init.show(types)).unwrap();
            }
            s
        }
    }
}

mod function_show {
    use crate::intu_ir::function;
    use crate::intu_ir::name::Name;
    use crate::intu_ir::types::Types;
    use std::fmt::Write;

    use super::Show;

    impl Show for function::Function {
        fn show(&self, types: &Types) -> String {
            let mut s = String::new();
            write!(s, "define ").unwrap();

            write!(s, "{} @{}(", self.return_type.show(types), self.name).unwrap();
            for (i, param) in self.parameters.iter().enumerate() {
                if i > 0 {
                    write!(s, ", ").unwrap();
                }
                write!(s, "{}", param.show(types)).unwrap();
            }

            write!(s, ")").unwrap();

            writeln!(s, " {{").unwrap();
            for bb in &self.basic_blocks {
                match &bb.name {
                    Name::Name(name) => write!(s, "{}:\n", name).unwrap(),
                    Name::Number(num) => write!(s, "{}:\n", num).unwrap(),
                }
                for instr in &bb.instrs {
                    writeln!(s, "  {}", instr.show(types)).unwrap();
                }
                writeln!(s, "  {}", bb.term.show(types)).unwrap();
            }
            write!(s, "}}").unwrap();

            s
        }
    }

    impl Show for function::FunctionDeclaration {
        fn show(&self, types: &Types) -> String {
            let mut s = String::new();
            write!(s, "declare ").unwrap();

            write!(s, " @{}(", self.name).unwrap();
            for (i, param) in self.parameters.iter().enumerate() {
                if i > 0 {
                    write!(s, ", ").unwrap();
                }
                write!(s, "{}", param.show(types)).unwrap();
            }
            writeln!(s, ")").unwrap();
            s
        }
    }

    impl Show for function::Parameter {
        fn show(&self, types: &Types) -> String {
            let mut s = String::new();
            write!(s, "{} {}", self.ty.show(types), self.name.show(types)).unwrap();
            s
        }
    }
}

mod operand_show {
    use crate::intu_ir::operand;
    use crate::intu_ir::operand::Operand;
    use crate::intu_ir::types::Types;
    use std::fmt::Write;

    use super::Show;

    impl Show for operand::Operand {
        fn show(&self, types: &Types) -> String {
            let mut s = String::new();
            match self {
                Operand::LocalOperand { name, ty: _ } => write!(s, "{}", name.show(types)).unwrap(),
                Operand::ConstantOperand(cref) => write!(s, "{}", cref.show(types)).unwrap(),
            }
            s
        }
    }
}

mod instruction_show {
    use crate::intu_ir::instruction;
    use crate::intu_ir::types::Types;
    use std::fmt::Write;

    use super::Show;
    impl Show for instruction::FPPredicate {
        fn show(&self, _types: &Types) -> String {
            match self {
                instruction::FPPredicate::False => "false".to_string(),
                instruction::FPPredicate::OEQ => "oeq".to_string(),
                instruction::FPPredicate::OGT => "ogt".to_string(),
                instruction::FPPredicate::OGE => "oge".to_string(),
                instruction::FPPredicate::OLT => "olt".to_string(),
                instruction::FPPredicate::OLE => "ole".to_string(),
                instruction::FPPredicate::ONE => "one".to_string(),
                instruction::FPPredicate::ORD => "ord".to_string(),
                instruction::FPPredicate::UNO => "uno".to_string(),
                instruction::FPPredicate::UEQ => "ueq".to_string(),
                instruction::FPPredicate::UGT => "ugt".to_string(),
                instruction::FPPredicate::UGE => "uge".to_string(),
                instruction::FPPredicate::ULT => "ult".to_string(),
                instruction::FPPredicate::ULE => "ule".to_string(),
                instruction::FPPredicate::UNE => "une".to_string(),
                instruction::FPPredicate::True => "true".to_string(),
            }
        }
    }

    impl Show for instruction::IntPredicate {
        fn show(&self, _types: &Types) -> String {
            match self {
                instruction::IntPredicate::EQ => "eq".to_string(),
                instruction::IntPredicate::NE => "ne".to_string(),
                instruction::IntPredicate::UGT => "ugt".to_string(),
                instruction::IntPredicate::UGE => "uge".to_string(),
                instruction::IntPredicate::ULT => "ult".to_string(),
                instruction::IntPredicate::ULE => "ule".to_string(),
                instruction::IntPredicate::SGT => "sgt".to_string(),
                instruction::IntPredicate::SGE => "sge".to_string(),
                instruction::IntPredicate::SLT => "slt".to_string(),
                instruction::IntPredicate::SLE => "sle".to_string(),
            }
        }
    }
    impl Show for instruction::Instruction {
        fn show(&self, types: &Types) -> String {
            use instruction::Instruction;
            match self {
                Instruction::Add { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(s, "{} = add", dest.show(types)).unwrap();

                    write!(s, " {} {}, {}", ty.show(types), operand0.show(types), operand1.show(types)).unwrap();
                    s
                },
                Instruction::Sub { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(s, "{} = sub", dest.show(types)).unwrap();

                    write!(s, " {} {}, {}", ty.show(types), operand0.show(types), operand1.show(types)).unwrap();
                    s
                },
                Instruction::Mul { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(s, "{} = mul", dest.show(types)).unwrap();

                    write!(s, " {} {}, {}", ty.show(types), operand0.show(types), operand1.show(types)).unwrap();
                    s
                },
                Instruction::UDiv { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(s, "{} = udiv", dest.show(types)).unwrap();

                    write!(s, " {} {}, {}", ty.show(types), operand0.show(types), operand1.show(types)).unwrap();
                    s
                },
                Instruction::SDiv { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(s, "{} = sdiv", dest.show(types)).unwrap();

                    write!(s, " {} {}, {}", ty.show(types), operand0.show(types), operand1.show(types)).unwrap();
                    s
                },
                Instruction::URem { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = urem {} {}, {}",
                        dest.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::SRem { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = srem {} {}, {}",
                        dest.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::And { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = and {} {}, {}",
                        dest.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::Or { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(s, "{} = or", dest.show(types)).unwrap();

                    write!(s, " {} {}, {}", ty.show(types), operand0.show(types), operand1.show(types)).unwrap();
                    s
                },
                Instruction::Xor { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = xor {} {}, {}",
                        dest.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::Shl { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(s, "{} = shl", dest.show(types)).unwrap();

                    write!(s, " {} {}, {}", ty.show(types), operand0.show(types), operand1.show(types)).unwrap();
                    s
                },
                Instruction::LShr { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(s, "{} = lshr", dest.show(types)).unwrap();

                    write!(s, " {} {}, {}", ty.show(types), operand0.show(types), operand1.show(types)).unwrap();
                    s
                },
                Instruction::AShr { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(s, "{} = ashr", dest.show(types)).unwrap();

                    write!(s, " {} {}, {}", ty.show(types), operand0.show(types), operand1.show(types)).unwrap();
                    s
                },
                Instruction::FAdd { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = fadd {} {}, {}",
                        dest.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::FSub { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = fsub {} {}, {}",
                        dest.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::FMul { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = fmul {} {}, {}",
                        dest.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::FDiv { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = fdiv {} {}, {}",
                        dest.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::FRem { operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = frem {} {}, {}",
                        dest.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::FNeg { operand, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand);
                    write!(s, "{} = fneg {} {}", dest.show(types), ty.show(types), operand.show(types)).unwrap();
                    s
                },
                Instruction::Alloca { allocated_type, num_elements, dest, alignment } => {
                    let mut s = String::new();
                    write!(s, "{} = alloca {}", dest.show(types), allocated_type.show(types)).unwrap();

                    write!(s, ", {} {}", types.type_of(num_elements).show(types), num_elements.show(types)).unwrap();

                    write!(s, ", align {}", alignment).unwrap();

                    s
                },
                Instruction::Load { address, dest, loaded_ty, alignment } => {
                    let mut s = String::new();
                    write!(s, "{} = load ", dest.show(types)).unwrap();
                    write!(
                        s,
                        "{}, {} {}",
                        loaded_ty.show(types),
                        types.type_of(address).show(types),
                        address.show(types)
                    )
                    .unwrap();

                    write!(s, ", align {}", alignment).unwrap();
                    s
                },
                Instruction::Store { address, value, alignment } => {
                    let mut s = String::new();
                    write!(s, "store ").unwrap();
                    write!(
                        s,
                        "{} {}, {} {}",
                        types.type_of(value).show(types),
                        value.show(types),
                        types.type_of(address).show(types),
                        address.show(types)
                    )
                    .unwrap();

                    write!(s, ", align {}", alignment).unwrap();
                    s
                },
                Instruction::GetElementPtr { address, indices, dest, source_element_type } => {
                    let mut s = String::new();
                    write!(s, "{} = getelementptr ", dest.show(types)).unwrap();
                    write!(
                        s,
                        "{}, {} {}",
                        source_element_type.show(types),
                        types.type_of(address).show(types),
                        address.show(types)
                    )
                    .unwrap();
                    for idx in indices {
                        write!(s, ", {} {}", types.type_of(idx).show(types), idx.show(types)).unwrap();
                    }
                    s
                },
                Instruction::Trunc { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = trunc {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::ZExt { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(s, "{} = zext", dest.show(types)).unwrap();

                    write!(s, " {} {} to {}", from_ty.show(types), operand.show(types), to_type.show(types)).unwrap();
                    s
                },
                Instruction::SExt { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = sext {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::FPTrunc { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = fptrunc {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::FPExt { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = fpext {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::FPToUI { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = fptoui {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::FPToSI { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = fptosi {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::UIToFP { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = uitofp {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::SIToFP { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = sitofp {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::PtrToInt { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = ptrtoint {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::IntToPtr { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = inttoptr {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::BitCast { operand, to_type, dest } => {
                    let mut s = String::new();
                    let from_ty = types.type_of(operand);
                    write!(
                        s,
                        "{} = bitcast {} {} to {}",
                        dest.show(types),
                        from_ty.show(types),
                        operand.show(types),
                        to_type.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::ICmp { predicate, operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = icmp {} {} {}, {}",
                        dest.show(types),
                        predicate.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::FCmp { predicate, operand0, operand1, dest } => {
                    let mut s = String::new();
                    let ty = types.type_of(operand0);
                    write!(
                        s,
                        "{} = fcmp {} {} {}, {}",
                        dest.show(types),
                        predicate.show(types),
                        ty.show(types),
                        operand0.show(types),
                        operand1.show(types)
                    )
                    .unwrap();
                    s
                },
                Instruction::Call { function, arguments, dest, is_tail_call, .. } => {
                    let mut s = String::new();
                    if let Some(dest) = dest {
                        write!(s, "{} = ", dest.show(types)).unwrap();
                    }
                    if *is_tail_call {
                        write!(s, "tail ").unwrap();
                    }
                    write!(s, "call {}(", format!("{} {}", types.type_of(self).show(types), function.show(types)),)
                        .unwrap();
                    for (i, arg) in arguments.iter().enumerate() {
                        if i == arguments.len() - 1 {
                            write!(s, "{}", arg.show(types)).unwrap();
                        } else {
                            write!(s, "{}, ", arg.show(types)).unwrap();
                        }
                    }
                    write!(s, ")").unwrap();
                    s
                },
                Instruction::ExtractValue { aggregate, indices, dest } => {
                    let mut s = String::new();
                    let agg_ty = types.type_of(aggregate);
                    write!(
                        s,
                        "{} = extractvalue {} {}, {}",
                        dest.show(types),
                        agg_ty.show(types),
                        aggregate.show(types),
                        indices.first().expect("ExtractValue with no indices")
                    )
                    .unwrap();
                    for idx in &indices[1..] {
                        write!(s, ", {idx}").unwrap();
                    }
                    s
                },
                Instruction::InsertValue { aggregate, element, indices, dest } => {
                    let mut s = String::new();
                    let agg_ty = types.type_of(aggregate);
                    write!(
                        s,
                        "{} = insertvalue {} {}, {}, {}",
                        dest.show(types),
                        agg_ty.show(types),
                        aggregate.show(types),
                        element.show(types),
                        indices.first().expect("InsertValue with no indices")
                    )
                    .unwrap();
                    for idx in &indices[1..] {
                        write!(s, ", {idx}").unwrap();
                    }
                    s
                },
            }
        }
    }
}

mod types_show {
    use crate::intu_ir::types;
    use crate::intu_ir::types::{NamedStructDef, Types};
    use std::fmt::Write;

    use super::Show;

    impl Show for types::NamedStructDef {
        fn show(&self, types: &Types) -> String {
            let mut s = String::new();
            match self {
                NamedStructDef::Opaque => write!(s, "type opaque").unwrap(),
                NamedStructDef::Defined(ty) => write!(s, "type {}", ty.show(types)).unwrap(),
            };
            s
        }
    }

    impl Show for types::InstType {
        fn show(&self, types: &Types) -> String {
            match self {
                types::InstType::VoidType => "void".to_string(),
                types::InstType::IntegerType { bits } => {
                    format!("i{}", bits)
                },
                types::InstType::PointerType { .. } => "ptr".to_string(),
                types::InstType::FPType(fpt) => fpt.show(types),
                types::InstType::FuncType { result_type, param_types } => {
                    let mut s = String::new();
                    write!(s, "{} (", result_type.show(types)).unwrap();
                    for (i, param_ty) in param_types.iter().enumerate() {
                        if i == param_types.len() - 1 {
                            write!(s, "{}", param_ty.show(types)).unwrap();
                        } else {
                            write!(s, "{}, ", param_ty.show(types)).unwrap();
                        }
                    }

                    write!(s, ")").unwrap();
                    s
                },
                types::InstType::VectorType { element_type, num_elements } => {
                    format!("<{} x {}>", num_elements, element_type.show(types))
                },
                types::InstType::ArrayType { element_type, num_elements } => {
                    format!("[{} x {}]", num_elements, element_type.show(types))
                },
                types::InstType::StructType { element_types } => {
                    let mut s = String::new();
                    write!(s, "{{ ").unwrap();
                    for (i, element_ty) in element_types.iter().enumerate() {
                        if i == element_types.len() - 1 {
                            write!(s, "{}", element_ty.show(types)).unwrap();
                        } else {
                            write!(s, "{}, ", element_ty.show(types)).unwrap();
                        }
                    }
                    write!(s, " }}").unwrap();
                    s
                },
                types::InstType::NamedStructType { name } => {
                    format!("%{}", name)
                },
            }
        }
    }

    impl Show for types::FPType {
        fn show(&self, _types: &Types) -> String {
            match self {
                types::FPType::Single => "float".to_string(),
                types::FPType::Double => "double".to_string(),
            }
        }
    }

    impl Show for types::TypeRef {
        fn show(&self, types: &Types) -> String {
            self.as_ref().show(types)
        }
    }
}

mod constant_show {
    use crate::intu_ir::constant;

    use crate::intu_ir::types::Types;
    use std::fmt::Write;

    use super::Show;

    impl Show for constant::Float {
        fn show(&self, _types: &Types) -> String {
            match self {
                constant::Float::Single(s) => format!("float {}", s),
                constant::Float::Double(d) => format!("double {}", d),
            }
        }
    }
    impl Show for constant::Constant {
        fn show(&self, types: &Types) -> String {
            let mut s = String::new();
            match self {
                constant::Constant::Int { bits, value } => {
                    if *bits == 1 {
                        if *value == 0 { write!(s, "false").unwrap() } else { write!(s, "true").unwrap() }
                    } else {
                        match *bits {
                            16 => write!(s, "{}", (*value & 0xFFFF) as i16).unwrap(),
                            32 => write!(s, "{}", (*value & 0xFFFF_FFFF) as i32).unwrap(),
                            64 => write!(s, "{}", *value as i64).unwrap(),
                            _ => write!(s, "{}", value).unwrap(),
                        }
                    }
                },
                constant::Constant::Float(f) => write!(s, "{}", f.show(types)).unwrap(),
                constant::Constant::Struct { name: _, values, is_packed } => {
                    if *is_packed {
                        write!(s, "<").unwrap();
                    }
                    write!(s, "{{ ").unwrap();
                    for (i, val) in values.iter().enumerate() {
                        if i == values.len() - 1 {
                            write!(s, "{}", val.show(types)).unwrap();
                        } else {
                            write!(s, "{}, ", val.show(types)).unwrap();
                        }
                    }
                    write!(s, " }}").unwrap();
                },
                constant::Constant::Array { element_type: _, elements } => {
                    write!(s, "[ ").unwrap();
                    for (i, elt) in elements.iter().enumerate() {
                        if i == elements.len() - 1 {
                            write!(s, "{}", elt.show(types)).unwrap();
                        } else {
                            write!(s, "{}, ", elt.show(types)).unwrap();
                        }
                    }
                    write!(s, " ]").unwrap();
                },
                constant::Constant::Vector(constant_refs) => {
                    write!(s, "< ").unwrap();
                    for (i, elt) in constant_refs.iter().enumerate() {
                        if i == constant_refs.len() - 1 {
                            write!(s, "{}", elt.show(types)).unwrap();
                        } else {
                            write!(s, "{}, ", elt.show(types)).unwrap();
                        }
                    }
                    write!(s, " >").unwrap();
                },
            }
            s
        }
    }
}

mod terminator_show {
    use crate::intu_ir::terminator;
    use crate::intu_ir::types::Types;
    use std::fmt::Write;

    use super::Show;

    impl Show for terminator::Terminator {
        fn show(&self, types: &Types) -> String {
            use terminator::Terminator;
            match self {
                Terminator::Ret { return_operand } => {
                    let mut s = String::new();
                    write!(s, "ret ").unwrap();
                    match return_operand {
                        None => write!(s, "void").unwrap(),
                        Some(op) => write!(s, "{} {}", types.type_of(op).show(types), op.show(types)).unwrap(),
                    }
                    s
                },
                Terminator::Br { dest } => {
                    format!("br label {}", dest.show(types))
                },
                Terminator::CondBr { condition, true_dest, false_dest } => {
                    let mut s = String::new();
                    write!(
                        s,
                        "br i1 {}, label {}, label {}",
                        condition.show(types),
                        true_dest.show(types),
                        false_dest.show(types)
                    )
                    .unwrap();
                    s
                },
                Terminator::IndirectBr { operand, possible_dests } => {
                    let mut s = String::new();
                    write!(
                        s,
                        "indirectbr {}, [ label {}",
                        operand.show(types),
                        possible_dests
                            .get(0)
                            .expect("IndirectBr with no possible dests")
                            .show(types)
                    )
                    .unwrap();
                    for dest in &possible_dests[1..] {
                        write!(s, ", label {}", dest.show(types)).unwrap();
                    }
                    write!(s, " ]").unwrap();
                    s
                },
                Terminator::Unreachable => "unreachable".to_string(),
            }
        }
    }
}

mod name_show {
    use super::Show;
    use crate::intu_ir::name;
    use crate::intu_ir::types::Types;
    use std::fmt::Write;

    impl Show for name::Name {
        fn show(&self, _types: &Types) -> String {
            let mut s = String::new();
            match self {
                name::Name::Name(name) => write!(s, "%{}", name).unwrap(),
                name::Name::Number(num) => write!(s, "%{}", num).unwrap(),
            };
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Show;
    use crate::intu_ir::basicblock::BasicBlock;
    use crate::intu_ir::constant::{Constant, ConstantRef, Float};
    use crate::intu_ir::function::{Function, FunctionDeclaration, Parameter};
    use crate::intu_ir::instruction::{FPPredicate, Instruction, IntPredicate};
    use crate::intu_ir::module::{GlobalVariable, Module};
    use crate::intu_ir::name::Name;
    use crate::intu_ir::operand::Operand;
    use crate::intu_ir::terminator::Terminator;
    use crate::intu_ir::types::{FPType, NamedStructDef, TypeRef, Types};

    fn mk_types() -> Types {
        Types::new()
    }

    fn mk_local(ty: TypeRef, name: &str) -> Operand {
        Operand::LocalOperand { name: Name::Name(name.into()), ty }
    }

    fn mk_const_int(bits: u32, value: u64) -> Operand {
        Operand::ConstantOperand(ConstantRef::new(Constant::Int { bits, value }))
    }

    fn mk_const_float_single(v: f32) -> Operand {
        Operand::ConstantOperand(ConstantRef::new(Constant::Float(Float::Single(v))))
    }

    fn mk_const_float_double(v: f64) -> Operand {
        Operand::ConstantOperand(ConstantRef::new(Constant::Float(Float::Double(v))))
    }

    // ========================= Name =========================

    #[test]
    fn test_name_named() {
        let types = mk_types();
        let name = Name::Name("foo".into());
        assert_eq!(name.show(&types), "%foo");
    }

    #[test]
    fn test_name_number() {
        let types = mk_types();
        let name = Name::Number(42);
        assert_eq!(name.show(&types), "%42");
    }

    // ========================= FPType =========================

    #[test]
    fn test_fptype_single() {
        let types = mk_types();
        assert_eq!(FPType::Single.show(&types), "float");
    }

    #[test]
    fn test_fptype_double() {
        let types = mk_types();
        assert_eq!(FPType::Double.show(&types), "double");
    }

    // ========================= InstType =========================

    fn show_insttype(ty: &TypeRef, types: &Types) -> String {
        ty.as_ref().show(types)
    }

    #[test]
    fn test_void_type() {
        let types = mk_types();
        assert_eq!(show_insttype(&types.void(), &types), "void");
    }

    #[test]
    fn test_integer_types() {
        let types = mk_types();
        assert_eq!(show_insttype(&types.bool(), &types), "i1");
        assert_eq!(show_insttype(&types.i8(), &types), "i8");
        assert_eq!(show_insttype(&types.i16(), &types), "i16");
        assert_eq!(show_insttype(&types.i32(), &types), "i32");
        assert_eq!(show_insttype(&types.i64(), &types), "i64");
    }

    #[test]
    fn test_pointer_type() {
        let types = mk_types();
        assert_eq!(show_insttype(&types.pointer(), &types), "ptr");
    }

    #[test]
    fn test_fp_type() {
        let types = mk_types();
        assert_eq!(show_insttype(&types.single(), &types), "float");
        assert_eq!(show_insttype(&types.double(), &types), "double");
    }

    #[test]
    fn test_func_type() {
        let types = mk_types();
        let void_fn = types.func_type(types.void(), vec![]);
        assert_eq!(show_insttype(&void_fn, &types), "void ()");

        let int_fn = types.func_type(types.i32(), vec![types.i32(), types.i64()]);
        assert_eq!(show_insttype(&int_fn, &types), "i32 (i32, i64)");

        let multi_param = types.func_type(types.double(), vec![types.single(), types.void()]);
        assert_eq!(show_insttype(&multi_param, &types), "double (float, void)");
    }

    #[test]
    fn test_vector_type() {
        let types = mk_types();
        let v = types.vector_of(types.i32(), 4);
        assert_eq!(show_insttype(&v, &types), "<4 x i32>");
    }

    #[test]
    fn test_array_type() {
        let types = mk_types();
        let arr = types.array_of(types.double(), 10);
        assert_eq!(show_insttype(&arr, &types), "[10 x double]");
    }

    #[test]
    fn test_struct_type() {
        let types = mk_types();
        let s = types.struct_of(vec![types.i32(), types.double(), types.pointer()]);
        assert_eq!(show_insttype(&s, &types), "{ i32, double, ptr }");
    }

    #[test]
    fn test_struct_type_empty() {
        let types = mk_types();
        let empty = types.struct_of(vec![]);
        assert_eq!(show_insttype(&empty, &types), "{  }");
    }

    #[test]
    fn test_named_struct_type() {
        let types = mk_types();
        let ns = types.named_struct("Foo");
        assert_eq!(show_insttype(&ns, &types), "%Foo");
    }

    // ========================= NamedStructDef =========================

    #[test]
    fn test_named_struct_def_opaque() {
        let types = mk_types();
        let def = NamedStructDef::Opaque;
        assert_eq!(def.show(&types), "type opaque");
    }

    #[test]
    fn test_named_struct_def_defined() {
        let types = mk_types();
        let inner = types.struct_of(vec![types.i32(), types.double()]);
        let def = NamedStructDef::Defined(inner);
        assert_eq!(def.show(&types), "type { i32, double }");
    }

    // ========================= Constant =========================

    #[test]
    fn test_constant_int() {
        let types = mk_types();
        let c = Constant::Int { bits: 32, value: 42 };
        assert_eq!(c.show(&types), "42");

        let c = Constant::Int { bits: 32, value: 0xFFFF_FFFF };
        assert_eq!(c.show(&types), "-1");

        let c_neg = Constant::Int { bits: 16, value: 0xFFFF };
        assert_eq!(c_neg.show(&types), "-1");
    }

    #[test]
    fn test_constant_int_i1_false() {
        let types = mk_types();
        let c = Constant::Int { bits: 1, value: 0 };
        assert_eq!(c.show(&types), "false");
    }

    #[test]
    fn test_constant_int_i1_true() {
        let types = mk_types();
        let c = Constant::Int { bits: 1, value: 1 };
        assert_eq!(c.show(&types), "true");
    }

    #[test]
    fn test_constant_float() {
        let types = mk_types();
        let c = Constant::Float(Float::Single(1.5));
        assert_eq!(c.show(&types), "float 1.5");

        let c = Constant::Float(Float::Double(-2.5));
        assert_eq!(c.show(&types), "double -2.5");
    }

    #[test]
    fn test_constant_struct() {
        let types = mk_types();
        let values = vec![
            ConstantRef::new(Constant::Int { bits: 32, value: 1 }),
            ConstantRef::new(Constant::Float(Float::Double(2.0))),
        ];
        let c = Constant::Struct { name: None, values, is_packed: false };
        assert_eq!(c.show(&types), "{ 1, double 2 }");
    }

    #[test]
    fn test_constant_array() {
        let types = mk_types();
        let elements = vec![
            ConstantRef::new(Constant::Int { bits: 32, value: 1 }),
            ConstantRef::new(Constant::Int { bits: 32, value: 2 }),
            ConstantRef::new(Constant::Int { bits: 32, value: 3 }),
        ];
        let c = Constant::Array { element_type: types.i32(), elements };
        assert_eq!(c.show(&types), "[ 1, 2, 3 ]");
    }

    #[test]
    fn test_constant_vector() {
        let types = mk_types();
        let elements = vec![
            ConstantRef::new(Constant::Int { bits: 32, value: 4 }),
            ConstantRef::new(Constant::Int { bits: 32, value: 5 }),
        ];
        let c = Constant::Vector(elements);
        assert_eq!(c.show(&types), "< 4, 5 >");
    }

    // ========================= Operand =========================

    #[test]
    fn test_local_operand_show() {
        let types = mk_types();
        let op = mk_local(types.i32(), "x");
        assert_eq!(op.show(&types), "%x");
    }

    #[test]
    fn test_constant_operand_show() {
        let types = mk_types();
        let op = mk_const_int(64, 99);
        assert_eq!(op.show(&types), "99");
    }

    // ========================= IntPredicate =========================

    #[test]
    fn test_int_predicates() {
        let types = mk_types();
        assert_eq!(IntPredicate::EQ.show(&types), "eq");
        assert_eq!(IntPredicate::NE.show(&types), "ne");
        assert_eq!(IntPredicate::UGT.show(&types), "ugt");
        assert_eq!(IntPredicate::UGE.show(&types), "uge");
        assert_eq!(IntPredicate::ULT.show(&types), "ult");
        assert_eq!(IntPredicate::ULE.show(&types), "ule");
        assert_eq!(IntPredicate::SGT.show(&types), "sgt");
        assert_eq!(IntPredicate::SGE.show(&types), "sge");
        assert_eq!(IntPredicate::SLT.show(&types), "slt");
        assert_eq!(IntPredicate::SLE.show(&types), "sle");
    }

    // ========================= FPPredicate =========================

    #[test]
    fn test_fp_predicates() {
        let types = mk_types();
        assert_eq!(FPPredicate::False.show(&types), "false");
        assert_eq!(FPPredicate::OEQ.show(&types), "oeq");
        assert_eq!(FPPredicate::OGT.show(&types), "ogt");
        assert_eq!(FPPredicate::OGE.show(&types), "oge");
        assert_eq!(FPPredicate::OLT.show(&types), "olt");
        assert_eq!(FPPredicate::OLE.show(&types), "ole");
        assert_eq!(FPPredicate::ONE.show(&types), "one");
        assert_eq!(FPPredicate::ORD.show(&types), "ord");
        assert_eq!(FPPredicate::UNO.show(&types), "uno");
        assert_eq!(FPPredicate::UEQ.show(&types), "ueq");
        assert_eq!(FPPredicate::UGT.show(&types), "ugt");
        assert_eq!(FPPredicate::UGE.show(&types), "uge");
        assert_eq!(FPPredicate::ULT.show(&types), "ult");
        assert_eq!(FPPredicate::ULE.show(&types), "ule");
        assert_eq!(FPPredicate::UNE.show(&types), "une");
        assert_eq!(FPPredicate::True.show(&types), "true");
    }

    // ========================= Instructions: binary arithmetic =========================

    #[test]
    fn test_add() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let b = mk_local(types.i32(), "b");
        let instr = Instruction::Add { operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = add i32 %a, %b");
    }

    #[test]
    fn test_sub() {
        let types = mk_types();
        let a = mk_local(types.i64(), "x");
        let b = mk_local(types.i64(), "y");
        let instr = Instruction::Sub { operand0: a, operand1: b, dest: Name::Number(1) };
        assert_eq!(instr.show(&types), "%1 = sub i64 %x, %y");
    }

    #[test]
    fn test_mul() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let b = mk_const_int(32, 3);
        let instr = Instruction::Mul { operand0: a, operand1: b, dest: Name::Number(2) };
        assert_eq!(instr.show(&types), "%2 = mul i32 %a, 3");
    }

    #[test]
    fn test_udiv() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let b = mk_local(types.i32(), "b");
        let instr = Instruction::UDiv { operand0: a, operand1: b, dest: Name::Name("q".into()) };
        assert_eq!(instr.show(&types), "%q = udiv i32 %a, %b");
    }

    #[test]
    fn test_sdiv() {
        let types = mk_types();
        let a = mk_local(types.i64(), "a");
        let b = mk_local(types.i64(), "b");
        let instr = Instruction::SDiv { operand0: a, operand1: b, dest: Name::Name("q".into()) };
        assert_eq!(instr.show(&types), "%q = sdiv i64 %a, %b");
    }

    #[test]
    fn test_urem() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let b = mk_local(types.i32(), "b");
        let instr = Instruction::URem { operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = urem i32 %a, %b");
    }

    #[test]
    fn test_srem() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let b = mk_local(types.i32(), "b");
        let instr = Instruction::SRem { operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = srem i32 %a, %b");
    }

    // ========================= Instructions: bitwise =========================

    #[test]
    fn test_and() {
        let types = mk_types();
        let a = mk_local(types.i8(), "x");
        let b = mk_const_int(8, 0x0F);
        let instr = Instruction::And { operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = and i8 %x, 15");
    }

    #[test]
    fn test_or() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let b = mk_local(types.i32(), "b");
        let instr = Instruction::Or { operand0: a, operand1: b, dest: Name::Number(3) };
        assert_eq!(instr.show(&types), "%3 = or i32 %a, %b");
    }

    #[test]
    fn test_xor() {
        let types = mk_types();
        let a = mk_local(types.i64(), "a");
        let b = mk_local(types.i64(), "b");
        let instr = Instruction::Xor { operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = xor i64 %a, %b");
    }

    // ========================= Instructions: shift =========================

    #[test]
    fn test_shl() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let b = mk_const_int(32, 2);
        let instr = Instruction::Shl { operand0: a, operand1: b, dest: Name::Number(4) };
        assert_eq!(instr.show(&types), "%4 = shl i32 %a, 2");
    }

    #[test]
    fn test_lshr() {
        let types = mk_types();
        let a = mk_local(types.i64(), "a");
        let b = mk_const_int(64, 3);
        let instr = Instruction::LShr { operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = lshr i64 %a, 3");
    }

    #[test]
    fn test_ashr() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let b = mk_const_int(32, 1);
        let instr = Instruction::AShr { operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = ashr i32 %a, 1");
    }

    // ========================= Instructions: float arithmetic =========================

    #[test]
    fn test_fadd() {
        let types = mk_types();
        let a = mk_local(types.double(), "a");
        let b = mk_local(types.double(), "b");
        let instr = Instruction::FAdd { operand0: a, operand1: b, dest: Name::Number(0) };
        assert_eq!(instr.show(&types), "%0 = fadd double %a, %b");
    }

    #[test]
    fn test_fsub() {
        let types = mk_types();
        let a = mk_local(types.single(), "x");
        let b = mk_const_float_single(1.0);
        let instr = Instruction::FSub { operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = fsub float %x, float 1");
    }

    #[test]
    fn test_fmul() {
        let types = mk_types();
        let a = mk_local(types.double(), "a");
        let b = mk_const_float_double(2.0);
        let instr = Instruction::FMul { operand0: a, operand1: b, dest: Name::Number(1) };
        assert_eq!(instr.show(&types), "%1 = fmul double %a, double 2");
    }

    #[test]
    fn test_fdiv() {
        let types = mk_types();
        let a = mk_local(types.double(), "a");
        let b = mk_local(types.double(), "b");
        let instr = Instruction::FDiv { operand0: a, operand1: b, dest: Name::Name("q".into()) };
        assert_eq!(instr.show(&types), "%q = fdiv double %a, %b");
    }

    #[test]
    fn test_frem() {
        let types = mk_types();
        let a = mk_local(types.double(), "a");
        let b = mk_const_float_double(3.0);
        let instr = Instruction::FRem { operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = frem double %a, double 3");
    }

    #[test]
    fn test_fneg() {
        let types = mk_types();
        let a = mk_local(types.double(), "a");
        let instr = Instruction::FNeg { operand: a, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = fneg double %a");
    }

    // ========================= Instructions: memory =========================

    #[test]
    fn test_alloca() {
        let types = mk_types();
        let num = mk_const_int(32, 1);
        let instr = Instruction::Alloca {
            allocated_type: types.i32(),
            num_elements: num,
            dest: Name::Name("p".into()),
            alignment: 4,
        };
        assert_eq!(instr.show(&types), "%p = alloca i32, i32 1, align 4");
    }

    #[test]
    fn test_load() {
        let types = mk_types();
        let addr = mk_local(types.pointer(), "p");
        let instr =
            Instruction::Load { address: addr, dest: Name::Name("v".into()), loaded_ty: types.i32(), alignment: 4 };
        assert_eq!(instr.show(&types), "%v = load i32, ptr %p, align 4");
    }

    #[test]
    fn test_store() {
        let types = mk_types();
        let val = mk_local(types.i32(), "v");
        let addr = mk_local(types.pointer(), "p");
        let instr = Instruction::Store { value: val, address: addr, alignment: 8 };
        assert_eq!(instr.show(&types), "store i32 %v, ptr %p, align 8");
    }

    #[test]
    fn test_gep() {
        let types = mk_types();
        let addr = mk_local(types.pointer(), "p");
        let idx = mk_local(types.i64(), "i");
        let instr = Instruction::GetElementPtr {
            address: addr,
            indices: vec![idx],
            dest: Name::Name("r".into()),
            source_element_type: types.i32(),
        };
        assert_eq!(instr.show(&types), "%r = getelementptr i32, ptr %p, i64 %i");
    }

    #[test]
    fn test_gep_multi_index() {
        let types = mk_types();
        let addr = mk_local(types.pointer(), "p");
        let idx0 = mk_const_int(64, 0);
        let idx1 = mk_local(types.i64(), "i");
        let instr = Instruction::GetElementPtr {
            address: addr,
            indices: vec![idx0, idx1],
            dest: Name::Name("r".into()),
            source_element_type: types.i32(),
        };
        assert_eq!(instr.show(&types), "%r = getelementptr i32, ptr %p, i64 0, i64 %i");
    }

    // ========================= Instructions: conversions =========================

    #[test]
    fn test_trunc() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let instr = Instruction::Trunc { operand: a, to_type: types.i8(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = trunc i32 %a to i8");
    }

    #[test]
    fn test_zext() {
        let types = mk_types();
        let a = mk_local(types.i8(), "a");
        let instr = Instruction::ZExt { operand: a, to_type: types.i32(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = zext i8 %a to i32");
    }

    #[test]
    fn test_sext() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let instr = Instruction::SExt { operand: a, to_type: types.i64(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = sext i32 %a to i64");
    }

    #[test]
    fn test_fptrunc() {
        let types = mk_types();
        let a = mk_local(types.double(), "a");
        let instr = Instruction::FPTrunc { operand: a, to_type: types.single(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = fptrunc double %a to float");
    }

    #[test]
    fn test_fpext() {
        let types = mk_types();
        let a = mk_local(types.single(), "a");
        let instr = Instruction::FPExt { operand: a, to_type: types.double(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = fpext float %a to double");
    }

    #[test]
    fn test_fptoui() {
        let types = mk_types();
        let a = mk_local(types.double(), "a");
        let instr = Instruction::FPToUI { operand: a, to_type: types.i32(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = fptoui double %a to i32");
    }

    #[test]
    fn test_fptosi() {
        let types = mk_types();
        let a = mk_local(types.double(), "a");
        let instr = Instruction::FPToSI { operand: a, to_type: types.i64(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = fptosi double %a to i64");
    }

    #[test]
    fn test_uitofp() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let instr = Instruction::UIToFP { operand: a, to_type: types.double(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = uitofp i32 %a to double");
    }

    #[test]
    fn test_sitofp() {
        let types = mk_types();
        let a = mk_local(types.i64(), "a");
        let instr = Instruction::SIToFP { operand: a, to_type: types.single(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = sitofp i64 %a to float");
    }

    #[test]
    fn test_ptrtoint() {
        let types = mk_types();
        let a = mk_local(types.pointer(), "p");
        let instr = Instruction::PtrToInt { operand: a, to_type: types.i64(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = ptrtoint ptr %p to i64");
    }

    #[test]
    fn test_inttoptr() {
        let types = mk_types();
        let a = mk_local(types.i64(), "a");
        let instr = Instruction::IntToPtr { operand: a, to_type: types.pointer(), dest: Name::Name("p".into()) };
        assert_eq!(instr.show(&types), "%p = inttoptr i64 %a to ptr");
    }

    #[test]
    fn test_bitcast() {
        let types = mk_types();
        let a = mk_local(types.single(), "a");
        let instr = Instruction::BitCast { operand: a, to_type: types.i32(), dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = bitcast float %a to i32");
    }

    // ========================= Instructions: comparisons =========================

    #[test]
    fn test_icmp() {
        let types = mk_types();
        let a = mk_local(types.i32(), "a");
        let b = mk_local(types.i32(), "b");
        let instr =
            Instruction::ICmp { predicate: IntPredicate::SLT, operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = icmp slt i32 %a, %b");
    }

    #[test]
    fn test_icmp_with_const() {
        let types = mk_types();
        let a = mk_local(types.i64(), "a");
        let b = mk_const_int(64, 0);
        let instr = Instruction::ICmp { predicate: IntPredicate::EQ, operand0: a, operand1: b, dest: Name::Number(5) };
        assert_eq!(instr.show(&types), "%5 = icmp eq i64 %a, 0");
    }

    #[test]
    fn test_fcmp() {
        let types = mk_types();
        let a = mk_local(types.double(), "a");
        let b = mk_local(types.double(), "b");
        let instr =
            Instruction::FCmp { predicate: FPPredicate::OLT, operand0: a, operand1: b, dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = fcmp olt double %a, %b");
    }

    // ========================= Instructions: call =========================

    #[test]
    fn test_call_void() {
        let types = mk_types();
        let fn_ty = types.func_type(types.void(), vec![types.i32()]);
        let fn_name = Operand::LocalOperand { name: Name::Name("puts".into()), ty: fn_ty };
        let arg = mk_local(types.i32(), "arg");
        let instr = Instruction::Call {
            function: fn_name,
            function_ty: types.func_type(types.void(), vec![types.i32()]),
            arguments: vec![arg],
            dest: None,
            is_tail_call: false,
        };
        assert_eq!(instr.show(&types), "call void %puts(%arg)");
    }

    #[test]
    fn test_call_with_dest() {
        let types = mk_types();
        let fn_name = Operand::LocalOperand {
            name: Name::Name("add".into()),
            ty: types.func_type(types.i32(), vec![types.i32(), types.i32()]),
        };
        let a = mk_local(types.i32(), "a");
        let b = mk_local(types.i32(), "b");
        let instr = Instruction::Call {
            function: fn_name,
            function_ty: types.func_type(types.i32(), vec![types.i32(), types.i32()]),
            arguments: vec![a, b],
            dest: Some(Name::Name("r".into())),
            is_tail_call: false,
        };
        assert_eq!(instr.show(&types), "%r = call i32 %add(%a, %b)");
    }

    #[test]
    fn test_tail_call() {
        let types = mk_types();
        let fn_name =
            Operand::LocalOperand { name: Name::Name("foo".into()), ty: types.func_type(types.void(), vec![]) };
        let instr = Instruction::Call {
            function: fn_name,
            function_ty: types.func_type(types.void(), vec![]),
            arguments: vec![],
            dest: None,
            is_tail_call: true,
        };
        assert_eq!(instr.show(&types), "tail call void %foo()");
    }

    // ========================= Instructions: aggregate =========================

    #[test]
    fn test_extract_value() {
        let types = mk_types();
        let s_type = types.struct_of(vec![types.i32(), types.double()]);
        let agg = mk_local(s_type, "s");
        let instr = Instruction::ExtractValue { aggregate: agg, indices: vec![0], dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = extractvalue { i32, double } %s, 0");
    }

    #[test]
    fn test_extract_value_multi_index() {
        let types = mk_types();
        let inner = types.array_of(types.i32(), 4);
        let outer = types.struct_of(vec![types.double(), inner]);
        let agg = mk_local(outer, "s");
        let instr = Instruction::ExtractValue { aggregate: agg, indices: vec![1, 2], dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = extractvalue { double, [4 x i32] } %s, 1, 2");
    }

    #[test]
    fn test_insert_value() {
        let types = mk_types();
        let s_type = types.struct_of(vec![types.i32(), types.double()]);
        let agg = mk_local(s_type.clone(), "s");
        let elem = mk_local(types.i32(), "e");
        let instr =
            Instruction::InsertValue { aggregate: agg, element: elem, indices: vec![0], dest: Name::Name("r".into()) };
        assert_eq!(instr.show(&types), "%r = insertvalue { i32, double } %s, %e, 0");
    }

    #[test]
    fn test_insert_value_multi_index() {
        let types = mk_types();
        let arr_ty = types.array_of(types.i32(), 3);
        let agg = mk_local(arr_ty.clone(), "a");
        let elem = mk_local(types.i32(), "e");
        let instr = Instruction::InsertValue {
            aggregate: agg,
            element: elem,
            indices: vec![0, 1],
            dest: Name::Name("r".into()),
        };
        assert_eq!(instr.show(&types), "%r = insertvalue [3 x i32] %a, %e, 0, 1");
    }

    // ========================= Terminators =========================

    #[test]
    fn test_ret_void() {
        let types = mk_types();
        let term = Terminator::Ret { return_operand: None };
        assert_eq!(term.show(&types), "ret void");
    }

    #[test]
    fn test_ret_value() {
        let types = mk_types();
        let val = mk_local(types.i32(), "r");
        let term = Terminator::Ret { return_operand: Some(val) };
        assert_eq!(term.show(&types), "ret i32 %r");
    }

    #[test]
    fn test_ret_constant() {
        let types = mk_types();
        let val = mk_const_int(32, 42);
        let term = Terminator::Ret { return_operand: Some(val) };
        assert_eq!(term.show(&types), "ret i32 42");
    }

    #[test]
    fn test_br() {
        let types = mk_types();
        let term = Terminator::Br { dest: Name::Name("loop".into()) };
        assert_eq!(term.show(&types), "br label %loop");
    }

    #[test]
    fn test_condbr() {
        let types = mk_types();
        let cond = mk_local(types.bool(), "c");
        let term = Terminator::CondBr {
            condition: cond,
            true_dest: Name::Name("then".into()),
            false_dest: Name::Name("else".into()),
        };
        assert_eq!(term.show(&types), "br i1 %c, label %then, label %else");
    }

    #[test]
    fn test_indirectbr() {
        let types = mk_types();
        let op = mk_local(types.pointer(), "addr");
        let term = Terminator::IndirectBr {
            operand: op,
            possible_dests: vec![Name::Name("d1".into()), Name::Name("d2".into())],
        };
        assert_eq!(term.show(&types), "indirectbr %addr, [ label %d1, label %d2 ]");
    }

    #[test]
    fn test_unreachable() {
        let types = mk_types();
        let term = Terminator::Unreachable;
        assert_eq!(term.show(&types), "unreachable");
    }

    // ========================= Parameter =========================

    #[test]
    fn test_parameter() {
        let types = mk_types();
        let param = Parameter { name: Name::Name("a".into()), ty: types.i32() };
        assert_eq!(param.show(&types), "i32 %a");
    }

    #[test]
    fn test_parameter_ptr() {
        let types = mk_types();
        let param = Parameter { name: Name::Name("p".into()), ty: types.pointer() };
        assert_eq!(param.show(&types), "ptr %p");
    }

    // ========================= FunctionDeclaration =========================

    #[test]
    fn test_function_declaration() {
        let types = mk_types();
        let decl = FunctionDeclaration {
            name: "puts".into(),
            parameters: vec![Parameter { name: Name::Name("s".into()), ty: types.pointer() }],
            return_type: types.void(),
            alignment: 0,
        };
        assert_eq!(decl.show(&types), "declare  @puts(ptr %s)\n");
    }

    #[test]
    fn test_function_declaration_multi_params() {
        let types = mk_types();
        let decl = FunctionDeclaration {
            name: "add".into(),
            parameters: vec![
                Parameter { name: Name::Name("a".into()), ty: types.i32() },
                Parameter { name: Name::Name("b".into()), ty: types.i32() },
            ],
            return_type: types.i32(),
            alignment: 0,
        };
        assert_eq!(decl.show(&types), "declare  @add(i32 %a, i32 %b)\n");
    }

    // ========================= Function =========================

    #[test]
    fn test_function_empty() {
        let types = mk_types();
        let func = Function {
            name: "main".into(),
            parameters: vec![],
            return_type: types.i32(),
            basic_blocks: vec![BasicBlock {
                name: Name::Name("entry".into()),
                instrs: vec![],
                term: Terminator::Ret { return_operand: Some(mk_const_int(32, 0)) },
            }],
        };
        assert_eq!(func.show(&types), "define i32 @main() {\nentry:\n  ret i32 0\n}");
    }

    #[test]
    fn test_function_with_instrs() {
        let types = mk_types();
        let a_operand = mk_local(types.i32(), "a");
        let b_operand = mk_local(types.i32(), "b");
        let func = Function {
            name: "add".into(),
            parameters: vec![
                Parameter { name: Name::Name("a".into()), ty: types.i32() },
                Parameter { name: Name::Name("b".into()), ty: types.i32() },
            ],
            return_type: types.i32(),
            basic_blocks: vec![BasicBlock {
                name: Name::Name("entry".into()),
                instrs: vec![Instruction::Add { operand0: a_operand, operand1: b_operand, dest: Name::Number(0) }],
                term: Terminator::Ret {
                    return_operand: Some(Operand::LocalOperand { name: Name::Number(0), ty: types.i32() }),
                },
            }],
        };
        assert_eq!(
            func.show(&types),
            concat!(
                "define i32 @add(i32 %a, i32 %b) {\n",
                "entry:\n",
                "  %0 = add i32 %a, %b\n",
                "  ret i32 %0\n",
                "}"
            )
        );
    }

    #[test]
    fn test_function_multiple_blocks() {
        let types = mk_types();
        let cond = mk_local(types.bool(), "c");
        let func = Function {
            name: "choose".into(),
            parameters: vec![Parameter { name: Name::Name("c".into()), ty: types.bool() }],
            return_type: types.i32(),
            basic_blocks: vec![
                BasicBlock {
                    name: Name::Name("entry".into()),
                    instrs: vec![],
                    term: Terminator::CondBr {
                        condition: cond,
                        true_dest: Name::Name("then".into()),
                        false_dest: Name::Name("else".into()),
                    },
                },
                BasicBlock {
                    name: Name::Name("then".into()),
                    instrs: vec![],
                    term: Terminator::Ret { return_operand: Some(mk_const_int(32, 1)) },
                },
                BasicBlock {
                    name: Name::Name("else".into()),
                    instrs: vec![],
                    term: Terminator::Ret { return_operand: Some(mk_const_int(32, 2)) },
                },
            ],
        };
        let output = func.show(&types);
        assert!(output.starts_with("define i32 @choose(i1 %c) {\n"));
        assert!(output.contains("entry:\n  br i1 %c, label %then, label %else\n"));
        assert!(output.contains("then:\n  ret i32 1\n"));
        assert!(output.contains("else:\n  ret i32 2\n"));
        assert!(output.ends_with("}"));
    }

    // ========================= GlobalVariable =========================

    #[test]
    fn test_global_variable_constant_with_init() {
        let types = mk_types();
        let gv = GlobalVariable {
            name: Name::Name("x".into()),
            is_constant: true,
            ty: types.i32(),
            addr_space: 0,
            initializer: Some(ConstantRef::new(Constant::Int { bits: 32, value: 42 })),
        };
        assert_eq!(gv.show(&types), "@%x = constant i32 42");
    }

    #[test]
    fn test_global_variable_global_no_init() {
        let types = mk_types();
        let gv = GlobalVariable {
            name: Name::Name("y".into()),
            is_constant: false,
            ty: types.i32(),
            addr_space: 0,
            initializer: None,
        };
        assert_eq!(gv.show(&types), "@%y = global i32");
    }

    #[test]
    fn test_global_variable_with_addrspace() {
        let types = mk_types();
        let gv = GlobalVariable {
            name: Name::Name("buf".into()),
            is_constant: false,
            ty: types.array_of(types.i8(), 256),
            addr_space: 1,
            initializer: None,
        };
        assert_eq!(gv.show(&types), "@%buf = addrspace(1) global [256 x i8]");
    }

    // ========================= Module =========================

    #[test]
    fn test_module_empty() {
        let types = Types::new();
        let module = Module {
            name: "test".into(),
            source_file_name: "test.ll".into(),
            functions: vec![],
            func_declarations: vec![],
            global_vars: vec![],
            types: types.clone(),
        };
        assert_eq!(module.show(&types), "source_filename = \"test.ll\"\n");
    }

    #[test]
    fn test_module_with_global_and_function() {
        let mut types = Types::new();
        types.add_named_struct_def("Foo".into(), NamedStructDef::Opaque);

        let module = Module {
            name: "test".into(),
            source_file_name: "test.ll".into(),
            functions: vec![Function {
                name: "add".into(),
                parameters: vec![
                    Parameter { name: Name::Name("a".into()), ty: types.i32() },
                    Parameter { name: Name::Name("b".into()), ty: types.i32() },
                ],
                return_type: types.i32(),
                basic_blocks: vec![BasicBlock {
                    name: Name::Name("entry".into()),
                    instrs: vec![Instruction::Add {
                        operand0: Operand::LocalOperand { name: Name::Name("a".into()), ty: types.i32() },
                        operand1: Operand::LocalOperand { name: Name::Name("b".into()), ty: types.i32() },
                        dest: Name::Number(0),
                    }],
                    term: Terminator::Ret {
                        return_operand: Some(Operand::LocalOperand { name: Name::Number(0), ty: types.i32() }),
                    },
                }],
            }],
            func_declarations: vec![FunctionDeclaration {
                name: "puts".into(),
                parameters: vec![Parameter { name: Name::Name("s".into()), ty: types.pointer() }],
                return_type: types.void(),
                alignment: 0,
            }],
            global_vars: vec![GlobalVariable {
                name: Name::Name("count".into()),
                is_constant: false,
                ty: types.i32(),
                addr_space: 0,
                initializer: Some(ConstantRef::new(Constant::Int { bits: 32, value: 0 })),
            }],
            types: types.clone(),
        };
        let output = module.show(&types);
        assert!(output.starts_with("source_filename = \"test.ll\"\n"));
        assert!(output.contains("%Foo = type opaque"));
        assert!(output.contains("@%count = global i32 0"));
        assert!(output.contains("declare  @puts("));
        assert!(output.contains("define i32 @add(i32 %a, i32 %b) {"));
        assert!(output.ends_with('\n'));
    }
}
