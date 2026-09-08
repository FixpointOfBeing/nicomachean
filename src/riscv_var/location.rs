use crate::{
    riscv::rv64imfd_reg::{FReg, XReg},
    syntax::Ident,
};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub enum RvVarLocation {
    Dummy(String),
    XVar(Ident),
    FVar(Ident),
    XReg(XReg),
    FReg(FReg),
    StackSlot { offset: i32, size: u32 },
}

#[inline]
pub fn fvar(s: String) -> RvVarLocation {
    RvVarLocation::FVar(s)
}

#[inline]
pub fn var(s: String) -> RvVarLocation {
    RvVarLocation::XVar(s)
}

#[inline]
pub fn x(n: u8) -> RvVarLocation {
    RvVarLocation::XReg(XReg::from_u8(n))
}

#[inline]
pub fn x0() -> RvVarLocation {
    x(0)
}

#[inline]
pub fn zero() -> RvVarLocation {
    RvVarLocation::XReg(XReg::zero())
}

#[inline]
pub fn ra() -> RvVarLocation {
    RvVarLocation::XReg(XReg::ra())
}

#[inline]
pub fn sp() -> RvVarLocation {
    RvVarLocation::XReg(XReg::sp())
}

#[inline]
pub fn gp() -> RvVarLocation {
    RvVarLocation::XReg(XReg::gp())
}

#[inline]
pub fn tp() -> RvVarLocation {
    RvVarLocation::XReg(XReg::tp())
}

#[inline]
pub fn t0() -> RvVarLocation {
    RvVarLocation::XReg(XReg::t0())
}

#[inline]
pub fn t1() -> RvVarLocation {
    RvVarLocation::XReg(XReg::t1())
}

#[inline]
pub fn t2() -> RvVarLocation {
    RvVarLocation::XReg(XReg::t2())
}

#[inline]
pub fn s0() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s0())
}

#[inline]
pub fn fp() -> RvVarLocation {
    RvVarLocation::XReg(XReg::fp())
}

#[inline]
pub fn s1() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s1())
}

#[inline]
pub fn a0() -> RvVarLocation {
    RvVarLocation::XReg(XReg::a0())
}

#[inline]
pub fn a1() -> RvVarLocation {
    RvVarLocation::XReg(XReg::a1())
}

#[inline]
pub fn a2() -> RvVarLocation {
    RvVarLocation::XReg(XReg::a2())
}

#[inline]
pub fn a3() -> RvVarLocation {
    RvVarLocation::XReg(XReg::a3())
}

#[inline]
pub fn a4() -> RvVarLocation {
    RvVarLocation::XReg(XReg::a4())
}

#[inline]
pub fn a5() -> RvVarLocation {
    RvVarLocation::XReg(XReg::a5())
}

#[inline]
pub fn a6() -> RvVarLocation {
    RvVarLocation::XReg(XReg::a6())
}

#[inline]
pub fn a7() -> RvVarLocation {
    RvVarLocation::XReg(XReg::a7())
}

#[inline]
pub fn s2() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s2())
}

#[inline]
pub fn s3() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s3())
}

#[inline]
pub fn s4() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s4())
}

#[inline]
pub fn s5() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s5())
}

#[inline]
pub fn s6() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s6())
}

#[inline]
pub fn s7() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s7())
}

#[inline]
pub fn s8() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s8())
}

#[inline]
pub fn s9() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s9())
}

#[inline]
pub fn s10() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s10())
}

#[inline]
pub fn s11() -> RvVarLocation {
    RvVarLocation::XReg(XReg::s11())
}

#[inline]
pub fn t3() -> RvVarLocation {
    RvVarLocation::XReg(XReg::t3())
}

#[inline]
pub fn t4() -> RvVarLocation {
    RvVarLocation::XReg(XReg::t4())
}

#[inline]
pub fn t5() -> RvVarLocation {
    RvVarLocation::XReg(XReg::t5())
}

#[inline]
pub fn t6() -> RvVarLocation {
    RvVarLocation::XReg(XReg::t6())
}

#[inline]
pub fn ft0() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft0())
}
#[inline]
pub fn ft1() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft1())
}

#[inline]
pub fn ft2() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft2())
}

#[inline]
pub fn ft3() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft3())
}

#[inline]
pub fn ft4() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft4())
}

#[inline]
pub fn ft5() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft5())
}

#[inline]
pub fn ft6() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft6())
}

#[inline]
pub fn ft7() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft7())
}

#[inline]
pub fn fs0() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs0())
}

#[inline]
pub fn fs1() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs1())
}

#[inline]
pub fn fa0() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fa0())
}

#[inline]
pub fn fa1() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fa1())
}

#[inline]
pub fn fa2() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fa2())
}

#[inline]
pub fn fa3() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fa3())
}

#[inline]
pub fn fa4() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fa4())
}

#[inline]
pub fn fa5() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fa5())
}

#[inline]
pub fn fa6() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fa6())
}

#[inline]
pub fn fa7() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fa7())
}

#[inline]
pub fn fs2() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs2())
}

#[inline]
pub fn fs3() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs3())
}

#[inline]
pub fn fs4() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs4())
}

#[inline]
pub fn fs5() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs5())
}

#[inline]
pub fn fs6() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs6())
}

#[inline]
pub fn fs7() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs7())
}

#[inline]
pub fn fs8() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs8())
}

#[inline]
pub fn fs9() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs9())
}

#[inline]
pub fn fs10() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs10())
}

#[inline]
pub fn fs11() -> RvVarLocation {
    RvVarLocation::FReg(FReg::fs11())
}

#[inline]
pub fn ft8() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft8())
}

#[inline]
pub fn ft9() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft9())
}

#[inline]
pub fn ft10() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft10())
}

#[inline]
pub fn ft11() -> RvVarLocation {
    RvVarLocation::FReg(FReg::ft11())
}

impl fmt::Display for RvVarLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RvVarLocation::XVar(name) => write!(f, "{}", name),
            RvVarLocation::XReg(reg) => write!(f, "{}", reg),
            RvVarLocation::FVar(name) => write!(f, "{}", name),
            RvVarLocation::FReg(reg) => write!(f, "{}", reg),
            RvVarLocation::Dummy(s) => write!(f, "${}", s),
            RvVarLocation::StackSlot { offset, .. } => write!(f, "{}(sp)", offset)
        }
    }
}
