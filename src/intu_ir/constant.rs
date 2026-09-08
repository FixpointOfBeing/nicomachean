use crate::intu_ir::types::{FPType, TypeRef, Typed, Types};
use std::ops::Deref;
use std::sync::Arc;

/// 常量（Constant）：编译期即可确定值的操作数。
#[derive(PartialEq, Clone, Debug, Hash)]
pub enum Constant {
    /// 整数常量：`i32 42`。`bits` 是位宽，`value` 是位模式（按无符号
    /// 存储，打印时按位宽解释成有符号数）。
    Int {
        bits: u32,
        value: u64,
    },
    /// 浮点常量：`double 1.5`，见 [`Float`]。
    Float(Float),
    /// 结构体常量：`{ i32, i32 } { i32 1, i32 2 }`。
    Struct {
        name: Option<String>,
        values: Vec<ConstantRef>,
        is_packed: bool,
    },
    /// 数组常量：`[2 x i32] [i32 1, i32 2]`。
    Array {
        element_type: TypeRef,
        elements: Vec<ConstantRef>,
    },
    Vector(Vec<ConstantRef>),
}

/// 浮点常量
#[derive(PartialEq, Clone, Debug)]
#[allow(non_camel_case_types)]
pub enum Float {
    /// 32 位单精度浮点（`float`，）。
    Single(f32),
    /// 64 位双精度浮点（`double`）。
    Double(f64),
}

impl std::hash::Hash for Float {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Float::Single(f) => ordered_float::OrderedFloat(*f).hash(state),
            Float::Double(f) => ordered_float::OrderedFloat(*f).hash(state),
        }
    }
}

impl Typed for Float {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.fp(match self {
            Float::Single(_) => FPType::Single,
            Float::Double(_) => FPType::Double,
        })
    }
}

impl Typed for Constant {
    fn get_type(&self, types: &Types) -> TypeRef {
        match self {
            Constant::Int { bits, .. } => types.int(*bits),
            Constant::Float(f) => types.type_of(f),
            Constant::Struct { values, .. } => types.struct_of(values.iter().map(|v| types.type_of(v)).collect()),
            Constant::Array { element_type, elements } => types.array_of(element_type.clone(), elements.len()),
            Constant::Vector(v) => types.vector_of(types.type_of(&v[0]), v.len()),
        }
    }
}

#[derive(PartialEq, Clone, Debug, Hash)]
pub struct ConstantRef(Arc<Constant>);

impl AsRef<Constant> for ConstantRef {
    fn as_ref(&self) -> &Constant {
        self.0.as_ref()
    }
}

impl Deref for ConstantRef {
    type Target = Constant;

    fn deref(&self) -> &Constant {
        self.0.deref()
    }
}

impl Typed for ConstantRef {
    fn get_type(&self, types: &Types) -> TypeRef {
        self.as_ref().get_type(types)
    }
}

impl ConstantRef {
    pub fn new(c: Constant) -> Self {
        Self(Arc::new(c))
    }
}
