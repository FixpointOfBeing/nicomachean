use crate::intu_ir::name::Name;
use crate::intu_ir::operand::Operand;
use crate::intu_ir::types::{InstType, TypeRef, Typed, Types};
use std::fmt::Debug;

///
/// - 整数二元运算：[`Add`]、[`Sub`]、[`Mul`]、[`UDiv`]、[`SDiv`]、
///   [`URem`]、[`SRem`]、[`And`]、[`Or`]、[`Xor`]、[`Shl`]、[`LShr`]、[`AShr`]
/// - 浮点运算：[`FAdd`]、[`FSub`]、[`FMul`]、[`FDiv`]、[`FRem`]、[`FNeg`]
/// - 聚合体（结构体/数组）运算：[`ExtractValue`]、[`InsertValue`]
/// - 内存访问：[`Alloca`]、[`Load`]、[`Store`]
/// - 类型转换：[`Trunc`]、[`ZExt`]、[`SExt`]、[`FPTrunc`]、[`FPExt`]、
///   [`FPToUI`]、[`FPToSI`]、[`UIToFP`]、[`SIToFP`]、[`PtrToInt`]、
///   [`IntToPtr`]、[`BitCast`]
/// - 比较：[`ICmp`]、[`FCmp`]
///
#[derive(PartialEq, Clone, Debug, Hash)]
pub enum Instruction {
    /// `add` —— 整数加法。
    ///
    /// 语法：
    /// ```text
    /// %dest = add i32 %a, %b
    Add { operand0: Operand, operand1: Operand, dest: Name },
    /// `sub` —— 整数减法。
    ///
    /// 语法：
    /// ```text
    /// %dest = sub i32 %a, %b
    Sub { operand0: Operand, operand1: Operand, dest: Name },
    /// `mul` —— 整数乘法。
    ///
    /// 语法：
    /// ```text
    /// %dest = mul i32 %a, %b
    Mul { operand0: Operand, operand1: Operand, dest: Name },
    /// `udiv` —— 无符号整数除法。
    ///
    /// 语法：
    /// ```text
    /// %dest = udiv i32 %a, %b
    UDiv { operand0: Operand, operand1: Operand, dest: Name },
    /// `sdiv` —— 有符号整数除法。
    ///
    /// 语法：
    /// ```text
    /// %dest = sdiv i32 %a, %b
    SDiv { operand0: Operand, operand1: Operand, dest: Name },
    /// `urem` —— 无符号整数取余。
    ///
    /// 语法：
    /// ```text
    /// %dest = urem i32 %a, %b
    URem { operand0: Operand, operand1: Operand, dest: Name },

    /// `srem` —— 有符号整数取余。
    ///
    /// 语法：
    /// ```text
    /// %dest = srem i32 %a, %b
    SRem { operand0: Operand, operand1: Operand, dest: Name },
    /// `and` —— 按位与。
    ///
    /// 语法：
    /// ```text
    /// %dest = and i32 %a, %b
    And { operand0: Operand, operand1: Operand, dest: Name },

    /// `or` —— 按位或。
    ///
    /// 语法：
    /// ```text
    /// %dest = or i32 %a, %b
    Or { operand0: Operand, operand1: Operand, dest: Name },
    /// `xor` —— 按位异或。
    ///
    /// 语法：
    /// ```text
    /// %dest = xor i32 %a, %b
    Xor { operand0: Operand, operand1: Operand, dest: Name },
    /// `shl` —— 逻辑左移。
    ///
    /// 语法：
    /// ```text
    /// %dest = shl i32 %a, %b
    Shl { operand0: Operand, operand1: Operand, dest: Name },
    /// `lshr` —— 逻辑右移：高位补 0。
    ///
    /// 语法：
    /// ```text
    /// %dest = lshr i32 %a, %b
    LShr { operand0: Operand, operand1: Operand, dest: Name },
    /// `ashr` —— 算术右移：高位补符号位。
    ///
    /// 语法：
    /// ```text
    /// %dest = ashr i32 %a, %b
    AShr { operand0: Operand, operand1: Operand, dest: Name },
    /// `fadd` —— 浮点加法。
    ///
    /// 语法：
    /// ```text
    /// %dest = fadd double %a, %b
    FAdd { operand0: Operand, operand1: Operand, dest: Name },
    /// `fsub` —— 浮点减法。
    ///
    /// 语法：
    /// ```text
    /// %dest = fsub double %a, %b
    FSub { operand0: Operand, operand1: Operand, dest: Name },
    /// `fmul` —— 浮点乘法。
    ///
    /// 语法：
    /// ```text
    /// %dest = fmul double %a, %b
    FMul { operand0: Operand, operand1: Operand, dest: Name },
    /// `fdiv` —— 浮点除法。
    ///
    /// 语法：
    /// ```text
    /// %dest = fdiv double %a, %b
    FDiv { operand0: Operand, operand1: Operand, dest: Name },
    /// `frem` —— 浮点取余（截断余数，语义同 C 的 `fmod`）。
    ///
    /// 语法：
    /// ```text
    /// %dest = frem double %a, %b
    FRem { operand0: Operand, operand1: Operand, dest: Name },
    /// `fneg` —— 浮点取负。
    ///
    /// 语法：
    /// ```text
    /// %dest = fneg double %a
    FNeg { operand: Operand, dest: Name },

    /// `extractvalue` —— 从聚合体（strut，array）中按索引路径提取子值，返回该子值副本。
    ///
    /// 语法：
    /// ```text
    /// %dest = extractvalue { i32, i32 } %agg, 1, 0
    /// ``
    ExtractValue { aggregate: Operand, indices: Vec<u32>, dest: Name },

    /// `insertvalue` —— 基于原聚合体（strut，array）生成新聚合体，将指定值插入到索引路径对应位置。
    ///
    /// 语法：
    /// ```text
    /// %dest = insertvalue { i32, i32 } %agg, i32 %val, 1, 0
    /// ```
    InsertValue { aggregate: Operand, element: Operand, indices: Vec<u32>, dest: Name },
    /// `alloca` —— 在栈上分配内存，返回指向新分配内存的指针。
    /// 语法：
    /// ```text
    /// %dest = alloca i32, align 4
    Alloca {
        /// 分配的元素类型（`alloca <type>`）。
        allocated_type: TypeRef,
        /// 元素个数；为 1 时通常省略（`alloca i32`）。
        num_elements: Operand,
        dest: Name,
        /// 对齐要求（`align N`）。
        alignment: u32,
    },
    /// `load` —— 从指针指向的内存读取一个值。
    ///
    /// 语法：
    /// ```text
    /// %dest = load i32, ptr %p, align
    Load { address: Operand, dest: Name, loaded_ty: TypeRef, alignment: u32 },
    /// `store` —— 把一个值写入指针指向的内存。
    ///
    /// 语法：
    /// ```text
    /// store i32 %v, ptr %p, align 4
    Store {
        address: Operand,
        value: Operand,
        /// 对齐要求（`align N`）。
        alignment: u32,
    },

    /// `getelementptr`（GEP）—— 指针运算：根据索引计算地址，不访问内存。
    ///
    /// 语法：
    /// ```text
    /// %dest = getelementptr i32, ptr %p, i64 %idx
    GetElementPtr {
        address: Operand,
        indices: Vec<Operand>,
        dest: Name,
        /// 指针指向的元素类型（opaque pointer 下必须显式给出，用于计算步长）。
        source_element_type: TypeRef,
    },
    /// `trunc` —— 整数截断：从宽整数截取低位，得到窄整数。
    ///
    /// 语法：
    /// ```text
    /// %dest = trunc i32 %a to i8
    Trunc { operand: Operand, to_type: TypeRef, dest: Name },
    /// `zext` —— 零扩展：窄整数高位补 0，扩展为宽整数。
    ///
    /// 语法：
    /// ```text
    /// %dest = zext i8 %a to i32
    ZExt { operand: Operand, to_type: TypeRef, dest: Name },
    /// `sext` —— 符号扩展：按源值的符号位填充高位。
    ///
    /// 语法：
    /// ```text
    /// %dest = sext i8 %a to i64
    SExt { operand: Operand, to_type: TypeRef, dest: Name },
    /// `fptrunc` —— 浮点降精度（如 `f64` → `f32`），可能损失精度，
    /// 溢出时得到无穷。
    ///
    /// 语法：
    /// ```text
    /// %dest = fptrunc double %a to float
    FPTrunc { operand: Operand, to_type: TypeRef, dest: Name },
    /// `fpext` —— 浮点升精度（如 `f32` → `f64`），无损。
    ///
    /// 语法：
    /// ```text
    /// %dest = fpext float %a to double
    /// ``
    FPExt { operand: Operand, to_type: TypeRef, dest: Name },
    /// `fptoui` —— 浮点转无符号整数（向零截断）。
    ///
    /// 语法：
    /// ```text
    /// %dest = fptoui double %a to i32
    FPToUI { operand: Operand, to_type: TypeRef, dest: Name },
    /// `fptosi` —— 浮点转有符号整数（向零截断）。
    ///
    /// 语法：
    /// ```text
    /// %dest = fptosi double %a to i32
    FPToSI { operand: Operand, to_type: TypeRef, dest: Name },
    /// `uitofp` —— 无符号整数转浮点（可能损失精度）。
    ///
    /// 语法：
    /// ```text
    /// %dest = uitofp i32 %a to double
    UIToFP { operand: Operand, to_type: TypeRef, dest: Name },

    /// `sitofp` —— 有符号整数转浮点（可能损失精度）。
    ///
    /// 语法：
    /// ```text
    /// %dest = sitofp i32 %a to double
    SIToFP { operand: Operand, to_type: TypeRef, dest: Name },
    /// `ptrtoint` —— 指针转整数，保留地址的位模式。
    ///
    /// 语法：
    /// ```text
    /// %dest = ptrtoint ptr %p to i64
    PtrToInt { operand: Operand, to_type: TypeRef, dest: Name },

    /// `inttoptr` —— 整数转指针（`ptrtoint` 的逆运算）。
    ///
    /// 语法：
    /// ```text
    /// %dest = inttoptr i64 %p to ptr
    IntToPtr { operand: Operand, to_type: TypeRef, dest: Name },
    /// `bitcast` —— 位重解释：保持底层位不变，只改变类型（大小必须相同）。
    ///
    /// 语法：
    /// ```text
    /// %dest = bitcast float %a to i32
    BitCast { operand: Operand, to_type: TypeRef, dest: Name },

    /// `icmp` —— 整数比较，结果为 `i1`（向量操作数时为 `<N x i1>`）。
    ///
    /// 语法：
    /// ```text
    /// %dest = icmp slt i32 %a, %b
    ICmp { predicate: IntPredicate, operand0: Operand, operand1: Operand, dest: Name },
    /// `fcmp` —— 浮点比较，结果为 `i1`（向量操作数时为 `<N x i1>`）。
    ///
    /// 语法：
    /// ```text
    /// %dest = fcmp olt double %a, %b
    FCmp { predicate: FPPredicate, operand0: Operand, operand1: Operand, dest: Name },
    /// `call` —— 调用函数。
    ///
    /// 语法：
    /// ```text
    /// %result = call i32 @foo(i32 %a)
    /// call void @bar()            // 返回 void 时没有 dest
    Call {
        function: Operand,
        function_ty: TypeRef,
        arguments: Vec<Operand>,
        /// 存放调用结果的名字；被调函数返回 `void` 时为 `None`。
        dest: Option<Name>, // will be None if the `function` returns void
        is_tail_call: bool,
    },
}

impl Typed for Instruction {
    fn get_type(&self, types: &Types) -> TypeRef {
        match self {
            Instruction::Add { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::Sub { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::Mul { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::UDiv { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::SDiv { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::URem { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::SRem { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::And { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::Or { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::Xor { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::Shl { operand0, .. } => types.type_of(operand0),
            Instruction::LShr { operand0, .. } => types.type_of(operand0),
            Instruction::AShr { operand0, .. } => types.type_of(operand0),
            Instruction::FAdd { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::FSub { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::FMul { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::FDiv { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::FRem { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                ty
            },
            Instruction::FNeg { operand, .. } => types.type_of(operand),
            Instruction::Alloca { .. } => types.pointer(),
            Instruction::Load { loaded_ty, .. } => loaded_ty.clone(),
            Instruction::Store { .. } => types.void(),
            Instruction::GetElementPtr { .. } => types.pointer(),
            Instruction::Trunc { to_type, .. } => to_type.clone(),
            Instruction::ZExt { to_type, .. } => to_type.clone(),
            Instruction::SExt { to_type, .. } => to_type.clone(),
            Instruction::FPTrunc { to_type, .. } => to_type.clone(),
            Instruction::FPExt { to_type, .. } => to_type.clone(),
            Instruction::FPToUI { to_type, .. } => to_type.clone(),
            Instruction::FPToSI { to_type, .. } => to_type.clone(),
            Instruction::UIToFP { to_type, .. } => to_type.clone(),
            Instruction::SIToFP { to_type, .. } => to_type.clone(),
            Instruction::PtrToInt { to_type, .. } => to_type.clone(),
            Instruction::IntToPtr { to_type, .. } => to_type.clone(),
            Instruction::BitCast { to_type, .. } => to_type.clone(),
            Instruction::ICmp { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                match ty.as_ref() {
                    InstType::VectorType { num_elements, .. } => types.vector_of(types.bool(), *num_elements),
                    _ => types.bool(),
                }
            },
            Instruction::FCmp { operand0, operand1, .. } => {
                let ty = types.type_of(operand0);
                debug_assert_eq!(ty, types.type_of(operand1));
                match ty.as_ref() {
                    InstType::VectorType { num_elements, .. } => types.vector_of(types.bool(), *num_elements),
                    _ => types.bool(),
                }
            },
            Instruction::Call { function_ty, .. } => match function_ty.as_ref() {
                InstType::FuncType { result_type, .. } => result_type.clone(),
                ty => panic!("Expected Call.function_ty to be a FuncType, got {:?}", ty),
            },
            Instruction::ExtractValue { aggregate, indices, .. } => {
                ev_type(types.type_of(aggregate), indices.iter().copied())
            },

            Instruction::InsertValue { aggregate, .. } => types.type_of(aggregate),
        }
    }
}

fn ev_type(cur_type: TypeRef, mut indices: impl Iterator<Item = u32>) -> TypeRef {
    match indices.next() {
        None => cur_type,
        Some(index) => match cur_type.as_ref() {
            InstType::ArrayType { element_type, .. } => ev_type(element_type.clone(), indices),
            InstType::StructType { element_types, .. } => ev_type(
                element_types
                    .get(index as usize)
                    .expect("ExtractValue index out of range")
                    .clone(),
                indices,
            ),
            _ => panic!("ExtractValue from something that's not ArrayType or StructType; its type is {:?}", cur_type),
        },
    }
}
impl Instruction {
    pub fn is_binary_op(&self) -> bool {
        match self {
            Instruction::Add { .. } => true,
            Instruction::Sub { .. } => true,
            Instruction::Mul { .. } => true,
            Instruction::UDiv { .. } => true,
            Instruction::SDiv { .. } => true,
            Instruction::URem { .. } => true,
            Instruction::SRem { .. } => true,
            Instruction::And { .. } => true,
            Instruction::Or { .. } => true,
            Instruction::Xor { .. } => true,
            Instruction::Shl { .. } => true,
            Instruction::LShr { .. } => true,
            Instruction::AShr { .. } => true,
            Instruction::FAdd { .. } => true,
            Instruction::FSub { .. } => true,
            Instruction::FMul { .. } => true,
            Instruction::FDiv { .. } => true,
            Instruction::FRem { .. } => true,
            _ => false,
        }
    }

    pub fn is_unary_op(&self) -> bool {
        match self {
            Instruction::BitCast { .. } => true,
            Instruction::FNeg { .. } => true,
            Instruction::FPExt { .. } => true,
            Instruction::FPToSI { .. } => true,
            Instruction::FPToUI { .. } => true,
            Instruction::FPTrunc { .. } => true,
            Instruction::IntToPtr { .. } => true,
            Instruction::PtrToInt { .. } => true,
            Instruction::SExt { .. } => true,
            Instruction::SIToFP { .. } => true,
            Instruction::Trunc { .. } => true,
            Instruction::UIToFP { .. } => true,
            Instruction::ZExt { .. } => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum IntPredicate {
    EQ,
    NE,
    UGT,
    UGE,
    ULT,
    ULE,
    SGT,
    SGE,
    SLT,
    SLE,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum FPPredicate {
    False,
    OEQ,
    OGT,
    OGE,
    OLT,
    OLE,
    ONE,
    ORD,
    UNO,
    UEQ,
    UGT,
    UGE,
    ULT,
    ULE,
    UNE,
    True,
}
