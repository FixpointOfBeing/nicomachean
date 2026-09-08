use crate::riscv::rv64imfd_imm::{Imm12, Imm13LowZeroBits1, Imm21LowZeroBits1, Imm32LowZeroBits12, Shamt5, Shamt6};
use crate::riscv::rv64imfd_reg::{FReg, XReg};
use std::fmt;

/// 浮点舍入模式
#[derive(Debug, Clone, PartialEq)]
pub enum Rm {
    /// 就近舍入，平局取偶（round to nearest, ties to even）
    Rne,
    /// 向零舍入（round towards zero）
    Rtz,
    /// 向下舍入，朝 -∞（round down）
    Rdn,
    /// 向上舍入，朝 +∞（round up）
    Rup,
    /// 就近舍入，平局取绝对值大者（round to nearest, ties to max magnitude）
    Rmm,
    /// 动态舍入模式，由 frm 寄存器决定
    Dyn,
}

impl fmt::Display for Rm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Rm::Rne => "rne",
            Rm::Rtz => "rtz",
            Rm::Rdn => "rdn",
            Rm::Rup => "rup",
            Rm::Rmm => "rmm",
            Rm::Dyn => "dyn",
        };
        f.write_str(name)
    }
}

/// RISC-V RV64 instruction
#[derive(Debug, Clone, PartialEq)]
pub enum RvInstr {
    // R-type
    Add {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Sub {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Sll {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Slt {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Sltu {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Xor {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Srl {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Sra {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Or {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    And {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },

    // R-type（M 扩展）
    /// 有符号乘法（取低 64 位）：rd = (rs1 *s rs2)[63:0]
    Mul {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 有符号乘法（取高 64 位）：rd = (rs1 *s rs2) >> 64
    Mulh {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 无符号乘法（取高 64 位）：rd = (rs1 *u rs2) >> 64
    Mulhu {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 有符号乘无符号（取高 64 位）：rd = (rs1 *s rs2_u) >> 64
    Mulhsu {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 有符号除法（向零截断）：rd = rs1 /s rs2；除零时 rd = -1，溢出（MIN / -1）时 rd = MIN
    Div {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 无符号除法：rd = rs1 /u rs2；除零时 rd = 2^64 - 1
    Divu {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 有符号取余：rd = rs1 %s rs2；除零时 rd = rs1，溢出（MIN % -1）时 rd = 0
    Rem {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 无符号取余：rd = rs1 %u rs2；除零时 rd = rs1
    Remu {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },

    // RV64 R-type W
    Addw {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Subw {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Sllw {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Srlw {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    Sraw {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },

    // RV64 R-type W（M 扩展）
    /// 32 位有符号乘法（取低 32 位），结果按 32 位符号扩展
    Mulw {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 32 位有符号除法（向零截断），结果按 32 位符号扩展；除零时 rd = -1，溢出时 rd = sext32(INT32_MIN)
    Divw {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 32 位无符号除法，结果按 32 位符号扩展；除零时 rd = 2^32 - 1
    Divuw {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 32 位有符号取余，结果按 32 位符号扩展；除零时 rd = rs1，溢出时 rd = 0
    Remw {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },
    /// 32 位无符号取余，结果按 32 位符号扩展；除零时 rd = rs1
    Remuw {
        rd: XReg,
        rs1: XReg,
        rs2: XReg,
    },

    // I-type
    Addi {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Slti {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Sltiu {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Xori {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Ori {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Andi {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Slli {
        rd: XReg,
        rs1: XReg,
        shamt: Shamt6,
    },
    Srli {
        rd: XReg,
        rs1: XReg,
        shamt: Shamt6,
    },
    Srai {
        rd: XReg,
        rs1: XReg,
        shamt: Shamt6,
    },

    // RV64 I-type W
    Addiw {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Slliw {
        rd: XReg,
        rs1: XReg,
        shamt: Shamt5,
    },
    Srliw {
        rd: XReg,
        rs1: XReg,
        shamt: Shamt5,
    },
    Sraiw {
        rd: XReg,
        rs1: XReg,
        shamt: Shamt5,
    },

    // Loads (I-type)
    Lb {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Lh {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Lw {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Ld {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Lbu {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Lhu {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Lwu {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },

    // Jalr (I-type)
    Jalr {
        rd: XReg,
        rs1: XReg,
        imm: Imm12,
    },

    // S-type
    Sb {
        rs2: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Sh {
        rs2: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Sw {
        rs2: XReg,
        rs1: XReg,
        imm: Imm12,
    },
    Sd {
        rs2: XReg,
        rs1: XReg,
        imm: Imm12,
    },

    // B-type
    Beq {
        rs1: XReg,
        rs2: XReg,
        imm: Imm13LowZeroBits1,
    },
    Bne {
        rs1: XReg,
        rs2: XReg,
        imm: Imm13LowZeroBits1,
    },
    Blt {
        rs1: XReg,
        rs2: XReg,
        imm: Imm13LowZeroBits1,
    },
    Bge {
        rs1: XReg,
        rs2: XReg,
        imm: Imm13LowZeroBits1,
    },
    Bltu {
        rs1: XReg,
        rs2: XReg,
        imm: Imm13LowZeroBits1,
    },
    Bgeu {
        rs1: XReg,
        rs2: XReg,
        imm: Imm13LowZeroBits1,
    },

    // Lui (U-type)
    Lui {
        rd: XReg,
        imm: Imm32LowZeroBits12,
    },

    // Auipc (U-type)
    Auipc {
        rd: XReg,
        imm: Imm32LowZeroBits12,
    },

    // Jal (J-type)
    Jal {
        rd: XReg,
        imm: Imm21LowZeroBits1,
    },

    // 浮点算术（带舍入模式）
    /// 单精度浮点加法：rd = rs1 + rs2（按 rm 舍入）
    FaddS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rm: Rm,
    },
    /// 双精度浮点加法：rd = rs1 + rs2（按 rm 舍入）
    FaddD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rm: Rm,
    },
    /// 单精度浮点减法：rd = rs1 - rs2（按 rm 舍入）
    FsubS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rm: Rm,
    },
    /// 双精度浮点减法：rd = rs1 - rs2（按 rm 舍入）
    FsubD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rm: Rm,
    },
    /// 单精度浮点乘法：rd = rs1 × rs2（按 rm 舍入）
    FmulS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rm: Rm,
    },
    /// 双精度浮点乘法：rd = rs1 × rs2（按 rm 舍入）
    FmulD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rm: Rm,
    },
    /// 单精度浮点除法：rd = rs1 / rs2（按 rm 舍入）
    FdivS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rm: Rm,
    },
    /// 双精度浮点除法：rd = rs1 / rs2（按 rm 舍入）
    FdivD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rm: Rm,
    },
    /// 单精度浮点平方根：rd = √rs1（按 rm 舍入）
    FsqrtS {
        rd: FReg,
        rs1: FReg,
        rm: Rm,
    },
    /// 双精度浮点平方根：rd = √rs1（按 rm 舍入）
    FsqrtD {
        rd: FReg,
        rs1: FReg,
        rm: Rm,
    },

    // 符号注入/拷贝（无舍入）
    /// 单精度拷贝符号位：rd = rs1 的绝对值与 rs2 的符号位组合
    FsgnjS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 双精度拷贝符号位：rd = rs1 的绝对值与 rs2 的符号位组合
    FsgnjD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 单精度拷贝相反符号位：rd = rs1 的绝对值与 ~rs2 的符号位组合
    FsgnjnS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 双精度拷贝相反符号位：rd = rs1 的绝对值与 ~rs2 的符号位组合
    FsgnjnD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 单精度符号位异或：rd = rs1 的绝对值与 (rs1 符号位 xor rs2 符号位) 组合
    FsgnjxS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 双精度符号位异或：rd = rs1 的绝对值与 (rs1 符号位 xor rs2 符号位) 组合
    FsgnjxD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
    },

    // 取最小值/最大值（无舍入，遵循 IEEE-754 2019 语义）
    /// 单精度取最小值：rd = min(rs1, rs2)
    FminS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 双精度取最小值：rd = min(rs1, rs2)
    FminD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 单精度取最大值：rd = max(rs1, rs2)
    FmaxS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 双精度取最大值：rd = max(rs1, rs2)
    FmaxD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
    },

    // 浮点比较（结果写入整数寄存器）
    /// 单精度浮点相等比较：rs1 == rs2 时 rd = 1，否则 rd = 0
    FeqS {
        rd: XReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 双精度浮点相等比较：rs1 == rs2 时 rd = 1，否则 rd = 0
    FeqD {
        rd: XReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 单精度浮点小于比较：rs1 < rs2 时 rd = 1，否则 rd = 0
    FltS {
        rd: XReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 双精度浮点小于比较：rs1 < rs2 时 rd = 1，否则 rd = 0
    FltD {
        rd: XReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 单精度浮点小于等于比较：rs1 <= rs2 时 rd = 1，否则 rd = 0
    FleS {
        rd: XReg,
        rs1: FReg,
        rs2: FReg,
    },
    /// 双精度浮点小于等于比较：rs1 <= rs2 时 rd = 1，否则 rd = 0
    FleD {
        rd: XReg,
        rs1: FReg,
        rs2: FReg,
    },

    // 寄存器移动（整数 ↔ 浮点，位模式不变）
    /// 将浮点寄存器的位模式按 32 位符号扩展移到整数寄存器：rd = sext32(bits(rs1))
    FmvXW {
        rd: XReg,
        rs1: FReg,
    },
    /// 将整数寄存器的低 32 位位模式移到浮点寄存器：rd = bits(rs1)[31:0]
    FmvWX {
        rd: FReg,
        rs1: XReg,
    },
    /// 将浮点寄存器的位模式（64 位）移到整数寄存器：rd = bits(rs1)
    FmvXD {
        rd: XReg,
        rs1: FReg,
    },
    /// 将整数寄存器的位模式（64 位）移到浮点寄存器：rd = bits(rs1)
    FmvDX {
        rd: FReg,
        rs1: XReg,
    },

    // 类型转换（整数 → 浮点，无舍入）
    /// 有符号整数转单精度浮点：rd = (float)rs1
    FcvtSW {
        rd: FReg,
        rs1: XReg,
    },
    /// 无符号整数转单精度浮点：rd = (float)rs1
    FcvtSWu {
        rd: FReg,
        rs1: XReg,
    },
    /// 有符号整数转双精度浮点：rd = (double)rs1
    FcvtDW {
        rd: FReg,
        rs1: XReg,
    },
    /// 无符号整数转双精度浮点：rd = (double)rs1
    FcvtDWu {
        rd: FReg,
        rs1: XReg,
    },

    // 类型转换（浮点 ↔ 浮点，无舍入）
    /// 单精度浮点转双精度浮点：rd = (double)rs1
    FcvtSD {
        rd: FReg,
        rs1: FReg,
    },
    /// 双精度浮点转单精度浮点：rd = (float)rs1
    FcvtDS {
        rd: FReg,
        rs1: FReg,
    },

    // 类型转换（浮点 → 整数，带舍入模式）
    /// 单精度浮点转有符号整数（按 rm 舍入，向零截断等价于 rtz）
    FcvtWS {
        rd: XReg,
        rs1: FReg,
        rm: Rm,
    },
    /// 单精度浮点转无符号整数（按 rm 舍入）
    FcvtWuS {
        rd: XReg,
        rs1: FReg,
        rm: Rm,
    },
    /// 双精度浮点转有符号整数（按 rm 舍入）
    FcvtWD {
        rd: XReg,
        rs1: FReg,
        rm: Rm,
    },
    /// 双精度浮点转无符号整数（按 rm 舍入）
    FcvtWuD {
        rd: XReg,
        rs1: FReg,
        rm: Rm,
    },

    // 融合乘加（R4-type，带舍入模式）
    /// 单精度融合乘加：rd = rs1×rs2 + rs3（按 rm 舍入）
    FmaddS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rs3: FReg,
        rm: Rm,
    },
    /// 单精度融合乘减：rd = rs1×rs2 - rs3（按 rm 舍入）
    FmsubS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rs3: FReg,
        rm: Rm,
    },
    /// 单精度融合负乘加：rd = -(rs1×rs2) + rs3（按 rm 舍入）
    FnmsubS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rs3: FReg,
        rm: Rm,
    },
    /// 单精度融合负乘减：rd = -(rs1×rs2) - rs3（按 rm 舍入）
    FnmaddS {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rs3: FReg,
        rm: Rm,
    },
    /// 双精度融合乘加：rd = rs1×rs2 + rs3（按 rm 舍入）
    FmaddD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rs3: FReg,
        rm: Rm,
    },
    /// 双精度融合乘减：rd = rs1×rs2 - rs3（按 rm 舍入）
    FmsubD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rs3: FReg,
        rm: Rm,
    },
    /// 双精度融合负乘加：rd = -(rs1×rs2) + rs3（按 rm 舍入）
    FnmsubD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rs3: FReg,
        rm: Rm,
    },
    /// 双精度融合负乘减：rd = -(rs1×rs2) - rs3（按 rm 舍入）
    FnmaddD {
        rd: FReg,
        rs1: FReg,
        rs2: FReg,
        rs3: FReg,
        rm: Rm,
    },

    // 浮点分类（结果写入整数寄存器）
    /// 单精度浮点分类：rd = 指示 rs1 类型的位掩码
    FclassS {
        rd: XReg,
        rs1: FReg,
    },
    /// 双精度浮点分类：rd = 指示 rs1 类型的位掩码
    FclassD {
        rd: XReg,
        rs1: FReg,
    },

    // RV64F/D 64 位整数转换（带舍入模式）
    /// 单精度浮点转有符号 64 位整数（按 rm 舍入）
    FcvtLS {
        rd: XReg,
        rs1: FReg,
        rm: Rm,
    },
    /// 单精度浮点转无符号 64 位整数（按 rm 舍入）
    FcvtLuS {
        rd: XReg,
        rs1: FReg,
        rm: Rm,
    },
    /// 有符号 64 位整数转单精度浮点（按 rm 舍入）
    FcvtSL {
        rd: FReg,
        rs1: XReg,
        rm: Rm,
    },
    /// 无符号 64 位整数转单精度浮点（按 rm 舍入）
    FcvtSLu {
        rd: FReg,
        rs1: XReg,
        rm: Rm,
    },
    /// 双精度浮点转有符号 64 位整数（按 rm 舍入）
    FcvtLD {
        rd: XReg,
        rs1: FReg,
        rm: Rm,
    },
    /// 双精度浮点转无符号 64 位整数（按 rm 舍入）
    FcvtLuD {
        rd: XReg,
        rs1: FReg,
        rm: Rm,
    },
    /// 有符号 64 位整数转双精度浮点（按 rm 舍入）
    FcvtDL {
        rd: FReg,
        rs1: XReg,
        rm: Rm,
    },
    /// 无符号 64 位整数转双精度浮点（按 rm 舍入）
    FcvtDLu {
        rd: FReg,
        rs1: XReg,
        rm: Rm,
    },

    // 浮点访存
    /// 加载单精度浮点：rd = Mem[rs1 + sext(imm)]
    Flw {
        rd: FReg,
        rs1: XReg,
        imm: Imm12,
    },
    /// 加载双精度浮点：rd = Mem[rs1 + sext(imm)]
    Fld {
        rd: FReg,
        rs1: XReg,
        imm: Imm12,
    },
    /// 存单精度浮点：Mem[rs1 + sext(imm)] = rs2
    Fsw {
        rs2: FReg,
        rs1: XReg,
        imm: Imm12,
    },
    /// 存双精度浮点：Mem[rs1 + sext(imm)] = rs2
    Fsd {
        rs2: FReg,
        rs1: XReg,
        imm: Imm12,
    },

    // Fence
    Fence {
        pred: u8,
        succ: u8,
    },
    FenceTso,

    // System instructions
    Ecall,
    Ebreak,

    // Unimplemented/illegal
    Unimp,
}

impl fmt::Display for RvInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add { rd, rs1, rs2 } => {
                write!(f, "add {rd}, {rs1}, {rs2}")
            },
            Self::Sub { rd, rs1, rs2 } => {
                write!(f, "sub {rd}, {rs1}, {rs2}")
            },
            Self::Sll { rd, rs1, rs2 } => {
                write!(f, "sll {rd}, {rs1}, {rs2}")
            },
            Self::Slt { rd, rs1, rs2 } => {
                write!(f, "slt {rd}, {rs1}, {rs2}")
            },
            Self::Sltu { rd, rs1, rs2 } => {
                write!(f, "sltu {rd}, {rs1}, {rs2}")
            },
            Self::Xor { rd, rs1, rs2 } => {
                write!(f, "xor {rd}, {rs1}, {rs2}")
            },
            Self::Srl { rd, rs1, rs2 } => {
                write!(f, "srl {rd}, {rs1}, {rs2}")
            },
            Self::Sra { rd, rs1, rs2 } => {
                write!(f, "sra {rd}, {rs1}, {rs2}")
            },
            Self::Or { rd, rs1, rs2 } => {
                write!(f, "or {rd}, {rs1}, {rs2}")
            },
            Self::And { rd, rs1, rs2 } => {
                write!(f, "and {rd}, {rs1}, {rs2}")
            },

            Self::Mul { rd, rs1, rs2 } => {
                write!(f, "mul {rd}, {rs1}, {rs2}")
            },
            Self::Mulh { rd, rs1, rs2 } => {
                write!(f, "mulh {rd}, {rs1}, {rs2}")
            },
            Self::Mulhu { rd, rs1, rs2 } => {
                write!(f, "mulhu {rd}, {rs1}, {rs2}")
            },
            Self::Mulhsu { rd, rs1, rs2 } => {
                write!(f, "mulhsu {rd}, {rs1}, {rs2}")
            },
            Self::Div { rd, rs1, rs2 } => {
                write!(f, "div {rd}, {rs1}, {rs2}")
            },
            Self::Divu { rd, rs1, rs2 } => {
                write!(f, "divu {rd}, {rs1}, {rs2}")
            },
            Self::Rem { rd, rs1, rs2 } => {
                write!(f, "rem {rd}, {rs1}, {rs2}")
            },
            Self::Remu { rd, rs1, rs2 } => {
                write!(f, "remu {rd}, {rs1}, {rs2}")
            },

            Self::Addw { rd, rs1, rs2 } => {
                write!(f, "addw {rd}, {rs1}, {rs2}")
            },
            Self::Subw { rd, rs1, rs2 } => {
                write!(f, "subw {rd}, {rs1}, {rs2}")
            },
            Self::Sllw { rd, rs1, rs2 } => {
                write!(f, "sllw {rd}, {rs1}, {rs2}")
            },
            Self::Srlw { rd, rs1, rs2 } => {
                write!(f, "srlw {rd}, {rs1}, {rs2}")
            },
            Self::Sraw { rd, rs1, rs2 } => {
                write!(f, "sraw {rd}, {rs1}, {rs2}")
            },

            Self::Mulw { rd, rs1, rs2 } => {
                write!(f, "mulw {rd}, {rs1}, {rs2}")
            },
            Self::Divw { rd, rs1, rs2 } => {
                write!(f, "divw {rd}, {rs1}, {rs2}")
            },
            Self::Divuw { rd, rs1, rs2 } => {
                write!(f, "divuw {rd}, {rs1}, {rs2}")
            },
            Self::Remw { rd, rs1, rs2 } => {
                write!(f, "remw {rd}, {rs1}, {rs2}")
            },
            Self::Remuw { rd, rs1, rs2 } => {
                write!(f, "remuw {rd}, {rs1}, {rs2}")
            },

            Self::Addi { rd, rs1, imm } => {
                write!(f, "addi {rd}, {rs1}, {imm}")
            },
            Self::Slti { rd, rs1, imm } => {
                write!(f, "slti {rd}, {rs1}, {imm}")
            },
            Self::Sltiu { rd, rs1, imm } => {
                write!(f, "sltiu {rd}, {rs1}, {imm}")
            },
            Self::Xori { rd, rs1, imm } => {
                write!(f, "xori {rd}, {rs1}, {imm}")
            },
            Self::Ori { rd, rs1, imm } => {
                write!(f, "ori {rd}, {rs1}, {imm}")
            },
            Self::Andi { rd, rs1, imm } => {
                write!(f, "andi {rd}, {rs1}, {imm}")
            },
            Self::Slli { rd, rs1, shamt } => {
                write!(f, "slli {rd}, {rs1}, {shamt}")
            },
            Self::Srli { rd, rs1, shamt } => {
                write!(f, "srli {rd}, {rs1}, {shamt}")
            },
            Self::Srai { rd, rs1, shamt } => {
                write!(f, "srai {rd}, {rs1}, {shamt}")
            },

            Self::Addiw { rd, rs1, imm } => {
                write!(f, "addiw {rd}, {rs1}, {imm}")
            },
            Self::Slliw { rd, rs1, shamt } => {
                write!(f, "slliw {rd}, {rs1}, {shamt}")
            },
            Self::Srliw { rd, rs1, shamt } => {
                write!(f, "srliw {rd}, {rs1}, {shamt}")
            },
            Self::Sraiw { rd, rs1, shamt } => {
                write!(f, "sraiw {rd}, {rs1}, {shamt}")
            },

            Self::Lb { rd, rs1, imm } => {
                write!(f, "lb {rd}, {imm}({rs1})")
            },
            Self::Lh { rd, rs1, imm } => {
                write!(f, "lh {rd}, {imm}({rs1})")
            },
            Self::Lw { rd, rs1, imm } => {
                write!(f, "lw {rd}, {imm}({rs1})")
            },
            Self::Ld { rd, rs1, imm } => {
                write!(f, "ld {rd}, {imm}({rs1})")
            },
            Self::Lbu { rd, rs1, imm } => {
                write!(f, "lbu {rd}, {imm}({rs1})")
            },
            Self::Lhu { rd, rs1, imm } => {
                write!(f, "lhu {rd}, {imm}({rs1})")
            },
            Self::Lwu { rd, rs1, imm } => {
                write!(f, "lwu {rd}, {imm}({rs1})")
            },

            Self::Jalr { rd, rs1, imm } => {
                write!(f, "jalr {rd}, {imm}({rs1})")
            },

            Self::Sb { rs2, rs1, imm } => {
                write!(f, "sb {rs2}, {imm}({rs1})")
            },
            Self::Sh { rs2, rs1, imm } => {
                write!(f, "sh {rs2}, {imm}({rs1})")
            },
            Self::Sw { rs2, rs1, imm } => {
                write!(f, "sw {rs2}, {imm}({rs1})")
            },
            Self::Sd { rs2, rs1, imm } => {
                write!(f, "sd {rs2}, {imm}({rs1})")
            },

            Self::Beq { rs1, rs2, imm } => {
                write!(f, "beq {rs1}, {rs2}, {imm}")
            },
            Self::Bne { rs1, rs2, imm } => {
                write!(f, "bne {rs1}, {rs2}, {imm}")
            },
            Self::Blt { rs1, rs2, imm } => {
                write!(f, "blt {rs1}, {rs2}, {imm}")
            },
            Self::Bge { rs1, rs2, imm } => {
                write!(f, "bge {rs1}, {rs2}, {imm}")
            },
            Self::Bltu { rs1, rs2, imm } => {
                write!(f, "bltu {rs1}, {rs2}, {imm}")
            },
            Self::Bgeu { rs1, rs2, imm } => {
                write!(f, "bgeu {rs1}, {rs2}, {imm}")
            },

            Self::Lui { rd, imm } => {
                write!(f, "lui {rd}, 0x{:x}", imm.to_i32() >> 12)
            },

            Self::Auipc { rd, imm } => {
                write!(f, "auipc {rd}, 0x{:x}", imm.to_i32() >> 12)
            },

            Self::Jal { rd, imm } => write!(f, "jal {rd}, {imm}"),

            Self::FaddS { rd, rs1, rs2, rm } => {
                write!(f, "fadd.s {rd}, {rs1}, {rs2}, {rm}")
            },
            Self::FaddD { rd, rs1, rs2, rm } => {
                write!(f, "fadd.d {rd}, {rs1}, {rs2}, {rm}")
            },
            Self::FsubS { rd, rs1, rs2, rm } => {
                write!(f, "fsub.s {rd}, {rs1}, {rs2}, {rm}")
            },
            Self::FsubD { rd, rs1, rs2, rm } => {
                write!(f, "fsub.d {rd}, {rs1}, {rs2}, {rm}")
            },
            Self::FmulS { rd, rs1, rs2, rm } => {
                write!(f, "fmul.s {rd}, {rs1}, {rs2}, {rm}")
            },
            Self::FmulD { rd, rs1, rs2, rm } => {
                write!(f, "fmul.d {rd}, {rs1}, {rs2}, {rm}")
            },
            Self::FdivS { rd, rs1, rs2, rm } => {
                write!(f, "fdiv.s {rd}, {rs1}, {rs2}, {rm}")
            },
            Self::FdivD { rd, rs1, rs2, rm } => {
                write!(f, "fdiv.d {rd}, {rs1}, {rs2}, {rm}")
            },
            Self::FsqrtS { rd, rs1, rm } => {
                write!(f, "fsqrt.s {rd}, {rs1}, {rm}")
            },
            Self::FsqrtD { rd, rs1, rm } => {
                write!(f, "fsqrt.d {rd}, {rs1}, {rm}")
            },

            Self::FsgnjS { rd, rs1, rs2 } => {
                write!(f, "fsgnj.s {rd}, {rs1}, {rs2}")
            },
            Self::FsgnjD { rd, rs1, rs2 } => {
                write!(f, "fsgnj.d {rd}, {rs1}, {rs2}")
            },
            Self::FsgnjnS { rd, rs1, rs2 } => {
                write!(f, "fsgnjn.s {rd}, {rs1}, {rs2}")
            },
            Self::FsgnjnD { rd, rs1, rs2 } => {
                write!(f, "fsgnjn.d {rd}, {rs1}, {rs2}")
            },
            Self::FsgnjxS { rd, rs1, rs2 } => {
                write!(f, "fsgnjx.s {rd}, {rs1}, {rs2}")
            },
            Self::FsgnjxD { rd, rs1, rs2 } => {
                write!(f, "fsgnjx.d {rd}, {rs1}, {rs2}")
            },
            Self::FminS { rd, rs1, rs2 } => {
                write!(f, "fmin.s {rd}, {rs1}, {rs2}")
            },
            Self::FminD { rd, rs1, rs2 } => {
                write!(f, "fmin.d {rd}, {rs1}, {rs2}")
            },
            Self::FmaxS { rd, rs1, rs2 } => {
                write!(f, "fmax.s {rd}, {rs1}, {rs2}")
            },
            Self::FmaxD { rd, rs1, rs2 } => {
                write!(f, "fmax.d {rd}, {rs1}, {rs2}")
            },

            Self::FeqS { rd, rs1, rs2 } => {
                write!(f, "feq.s {rd}, {rs1}, {rs2}")
            },
            Self::FeqD { rd, rs1, rs2 } => {
                write!(f, "feq.d {rd}, {rs1}, {rs2}")
            },
            Self::FltS { rd, rs1, rs2 } => {
                write!(f, "flt.s {rd}, {rs1}, {rs2}")
            },
            Self::FltD { rd, rs1, rs2 } => {
                write!(f, "flt.d {rd}, {rs1}, {rs2}")
            },
            Self::FleS { rd, rs1, rs2 } => {
                write!(f, "fle.s {rd}, {rs1}, {rs2}")
            },
            Self::FleD { rd, rs1, rs2 } => {
                write!(f, "fle.d {rd}, {rs1}, {rs2}")
            },

            Self::FmvXW { rd, rs1 } => {
                write!(f, "fmv.x.w {rd}, {rs1}")
            },
            Self::FmvWX { rd, rs1 } => {
                write!(f, "fmv.w.x {rd}, {rs1}")
            },
            Self::FmvXD { rd, rs1 } => {
                write!(f, "fmv.x.d {rd}, {rs1}")
            },
            Self::FmvDX { rd, rs1 } => {
                write!(f, "fmv.d.x {rd}, {rs1}")
            },

            Self::FcvtSW { rd, rs1 } => {
                write!(f, "fcvt.s.w {rd}, {rs1}")
            },
            Self::FcvtSWu { rd, rs1 } => {
                write!(f, "fcvt.s.wu {rd}, {rs1}")
            },
            Self::FcvtDW { rd, rs1 } => {
                write!(f, "fcvt.d.w {rd}, {rs1}")
            },
            Self::FcvtDWu { rd, rs1 } => {
                write!(f, "fcvt.d.wu {rd}, {rs1}")
            },
            Self::FcvtSD { rd, rs1 } => {
                write!(f, "fcvt.s.d {rd}, {rs1}")
            },
            Self::FcvtDS { rd, rs1 } => {
                write!(f, "fcvt.d.s {rd}, {rs1}")
            },
            Self::FcvtWS { rd, rs1, rm } => {
                write!(f, "fcvt.w.s {rd}, {rs1}, {rm}")
            },
            Self::FcvtWuS { rd, rs1, rm } => {
                write!(f, "fcvt.wu.s {rd}, {rs1}, {rm}")
            },
            Self::FcvtWD { rd, rs1, rm } => {
                write!(f, "fcvt.w.d {rd}, {rs1}, {rm}")
            },
            Self::FcvtWuD { rd, rs1, rm } => {
                write!(f, "fcvt.wu.d {rd}, {rs1}, {rm}")
            },

            Self::FmaddS { rd, rs1, rs2, rs3, rm } => {
                write!(f, "fmadd.s {rd}, {rs1}, {rs2}, {rs3}, {rm}")
            },
            Self::FmsubS { rd, rs1, rs2, rs3, rm } => {
                write!(f, "fmsub.s {rd}, {rs1}, {rs2}, {rs3}, {rm}")
            },
            Self::FnmsubS { rd, rs1, rs2, rs3, rm } => {
                write!(f, "fnmsub.s {rd}, {rs1}, {rs2}, {rs3}, {rm}")
            },
            Self::FnmaddS { rd, rs1, rs2, rs3, rm } => {
                write!(f, "fnmadd.s {rd}, {rs1}, {rs2}, {rs3}, {rm}")
            },
            Self::FmaddD { rd, rs1, rs2, rs3, rm } => {
                write!(f, "fmadd.d {rd}, {rs1}, {rs2}, {rs3}, {rm}")
            },
            Self::FmsubD { rd, rs1, rs2, rs3, rm } => {
                write!(f, "fmsub.d {rd}, {rs1}, {rs2}, {rs3}, {rm}")
            },
            Self::FnmsubD { rd, rs1, rs2, rs3, rm } => {
                write!(f, "fnmsub.d {rd}, {rs1}, {rs2}, {rs3}, {rm}")
            },
            Self::FnmaddD { rd, rs1, rs2, rs3, rm } => {
                write!(f, "fnmadd.d {rd}, {rs1}, {rs2}, {rs3}, {rm}")
            },
            Self::FclassS { rd, rs1 } => {
                write!(f, "fclass.s {rd}, {rs1}")
            },
            Self::FclassD { rd, rs1 } => {
                write!(f, "fclass.d {rd}, {rs1}")
            },
            Self::FcvtLS { rd, rs1, rm } => {
                write!(f, "fcvt.l.s {rd}, {rs1}, {rm}")
            },
            Self::FcvtLuS { rd, rs1, rm } => {
                write!(f, "fcvt.lu.s {rd}, {rs1}, {rm}")
            },
            Self::FcvtSL { rd, rs1, rm } => {
                write!(f, "fcvt.s.l {rd}, {rs1}, {rm}")
            },
            Self::FcvtSLu { rd, rs1, rm } => {
                write!(f, "fcvt.s.lu {rd}, {rs1}, {rm}")
            },
            Self::FcvtLD { rd, rs1, rm } => {
                write!(f, "fcvt.l.d {rd}, {rs1}, {rm}")
            },
            Self::FcvtLuD { rd, rs1, rm } => {
                write!(f, "fcvt.lu.d {rd}, {rs1}, {rm}")
            },
            Self::FcvtDL { rd, rs1, rm } => {
                write!(f, "fcvt.d.l {rd}, {rs1}, {rm}")
            },
            Self::FcvtDLu { rd, rs1, rm } => {
                write!(f, "fcvt.d.lu {rd}, {rs1}, {rm}")
            },

            Self::Flw { rd, rs1, imm } => {
                write!(f, "flw {rd}, {imm}({rs1})")
            },
            Self::Fld { rd, rs1, imm } => {
                write!(f, "fld {rd}, {imm}({rs1})")
            },
            Self::Fsw { rs2, rs1, imm } => {
                write!(f, "fsw {rs2}, {imm}({rs1})")
            },
            Self::Fsd { rs2, rs1, imm } => {
                write!(f, "fsd {rs2}, {imm}({rs1})")
            },

            Self::Fence { pred, succ } => {
                write!(f, "fence {pred}, {succ}")
            },
            Self::FenceTso => write!(f, "fence.tso"),

            Self::Ecall => write!(f, "ecall"),
            Self::Ebreak => write!(f, "ebreak"),

            Self::Unimp => write!(f, "unimp"),
        }
    }
}

/// 伪指令：单精度浮点拷贝 rd = rs，展开为 fsgnj.s rd, rs, rs
pub fn fmv_s(rd: FReg, rs: FReg) -> RvInstr {
    RvInstr::FsgnjS { rd: rd, rs1: rs, rs2: rs }
}

/// 伪指令：双精度浮点拷贝 rd = rs，展开为 fsgnj.d rd, rs, rs
pub fn fmv_d(rd: FReg, rs: FReg) -> RvInstr {
    RvInstr::FsgnjD { rd: rd, rs1: rs, rs2: rs }
}

/// 伪指令：单精度取负 rd = -rs，展开为 fsgnjn.s rd, rs, rs
pub fn fneg_s(rd: FReg, rs: FReg) -> RvInstr {
    RvInstr::FsgnjnS { rd: rd, rs1: rs, rs2: rs }
}

/// 伪指令：双精度取负 rd = -rs，展开为 fsgnjn.d rd, rs, rs
pub fn fneg_d(rd: FReg, rs: FReg) -> RvInstr {
    RvInstr::FsgnjnD { rd: rd, rs1: rs, rs2: rs }
}

/// 伪指令：单精度取绝对值 rd = |rs|，展开为 fsgnjx.s rd, rs, rs
pub fn fabs_s(rd: FReg, rs: FReg) -> RvInstr {
    RvInstr::FsgnjxS { rd: rd, rs1: rs, rs2: rs }
}

/// 伪指令：双精度取绝对值 rd = |rs|，展开为 fsgnjx.d rd, rs, rs
pub fn fabs_d(rd: FReg, rs: FReg) -> RvInstr {
    RvInstr::FsgnjxD { rd: rd, rs1: rs, rs2: rs }
}
