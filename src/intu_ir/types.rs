use crate::intu_ir::module::AddrSpace;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
#[allow(non_camel_case_types)]
pub enum InstType {
    VoidType,
    IntegerType { bits: u32 },
    PointerType { addr_space: AddrSpace },
    FPType(FPType),
    FuncType { result_type: TypeRef, param_types: Vec<TypeRef> },
    VectorType { element_type: TypeRef, num_elements: usize },
    ArrayType { element_type: TypeRef, num_elements: usize },
    StructType { element_types: Vec<TypeRef> },
    NamedStructType { name: String },
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum FPType {
    Single,
    Double,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct TypeRef(Arc<InstType>);

impl TypeRef {
    fn new(ty: InstType) -> Self {
        Self(Arc::new(ty))
    }
}

pub trait Typed {
    fn get_type(&self, types: &Types) -> TypeRef;
}

impl Typed for TypeRef {
    fn get_type(&self, _types: &Types) -> TypeRef {
        self.clone()
    }
}

impl Typed for InstType {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.get_for_type(self)
    }
}

impl Typed for FPType {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.fp(*self)
    }
}

#[derive(Clone, Debug, Hash)]
pub enum NamedStructDef {
    Opaque,
    Defined(TypeRef),
}

#[derive(Clone, Debug)]
pub struct Types {
    void_type: TypeRef,
    int_types: TypeCache<u32>,
    pointer_types: TypeCache<AddrSpace>,
    fp_types: TypeCache<FPType>,
    func_types: TypeCache<(TypeRef, Vec<TypeRef>)>,
    vec_types: TypeCache<(TypeRef, usize)>,
    arr_types: TypeCache<(TypeRef, usize)>,
    struct_types: TypeCache<Vec<TypeRef>>,
    named_struct_types: TypeCache<String>,
    named_struct_defs: HashMap<String, NamedStructDef>,
}

impl AsRef<InstType> for TypeRef {
    fn as_ref(&self) -> &InstType {
        self.0.as_ref()
    }
}

impl Deref for TypeRef {
    type Target = InstType;

    fn deref(&self) -> &InstType {
        self.0.deref()
    }
}
impl Types {
    pub fn new() -> Self {
        Self {
            void_type: TypeRef::new(InstType::VoidType),
            int_types: TypeCache::new(),
            pointer_types: TypeCache::new(),
            fp_types: TypeCache::new(),
            func_types: TypeCache::new(),
            vec_types: TypeCache::new(),
            arr_types: TypeCache::new(),
            struct_types: TypeCache::new(),
            named_struct_types: TypeCache::new(),
            named_struct_defs: HashMap::new(),
        }
    }
    pub fn type_of<T: Typed + ?Sized>(&self, t: &T) -> TypeRef {
        t.get_type(self)
    }

    pub fn void(&self) -> TypeRef {
        self.void_type.clone()
    }

    pub fn int(&self, bits: u32) -> TypeRef {
        self.int_types
            .lookup(&bits)
            .unwrap_or_else(|| TypeRef::new(InstType::IntegerType { bits }))
    }

    pub fn bool(&self) -> TypeRef {
        self.int(1)
    }

    pub fn i8(&self) -> TypeRef {
        self.int(8)
    }

    pub fn i16(&self) -> TypeRef {
        self.int(16)
    }

    pub fn i32(&self) -> TypeRef {
        self.int(32)
    }

    pub fn i64(&self) -> TypeRef {
        self.int(64)
    }

    pub fn pointer(&self) -> TypeRef {
        self.pointer_in_addr_space(0)
    }

    pub fn pointer_in_addr_space(&self, addr_space: AddrSpace) -> TypeRef {
        self.pointer_types
            .lookup(&addr_space)
            .unwrap_or_else(|| TypeRef::new(InstType::PointerType { addr_space }))
    }

    pub fn fp(&self, fpt: FPType) -> TypeRef {
        self.fp_types
            .lookup(&fpt)
            .unwrap_or_else(|| TypeRef::new(InstType::FPType(fpt)))
    }

    pub fn single(&self) -> TypeRef {
        self.fp(FPType::Single)
    }

    pub fn double(&self) -> TypeRef {
        self.fp(FPType::Double)
    }

    pub fn func_type(&self, result_type: TypeRef, param_types: Vec<TypeRef>) -> TypeRef {
        self.func_types
            .lookup(&(result_type.clone(), param_types.clone()))
            .unwrap_or_else(|| TypeRef::new(InstType::FuncType { result_type, param_types }))
    }

    pub fn vector_of(&self, element_type: TypeRef, num_elements: usize) -> TypeRef {
        self.vec_types
            .lookup(&(element_type.clone(), num_elements))
            .unwrap_or_else(|| TypeRef::new(InstType::VectorType { element_type, num_elements }))
    }

    pub fn array_of(&self, element_type: TypeRef, num_elements: usize) -> TypeRef {
        self.arr_types
            .lookup(&(element_type.clone(), num_elements))
            .unwrap_or_else(|| TypeRef::new(InstType::ArrayType { element_type, num_elements }))
    }

    pub fn struct_of(&self, element_types: Vec<TypeRef>) -> TypeRef {
        self.struct_types
            .lookup(&element_types.clone())
            .unwrap_or_else(|| TypeRef::new(InstType::StructType { element_types }))
    }

    pub fn named_struct(&self, name: &str) -> TypeRef {
        self.named_struct_types
            .lookup(name)
            .unwrap_or_else(|| TypeRef::new(InstType::NamedStructType { name: name.into() }))
    }

    pub fn named_struct_def(&self, name: &str) -> Option<&NamedStructDef> {
        self.named_struct_defs.get(name)
    }

    pub fn all_struct_names(&self) -> impl Iterator<Item = &String> {
        self.named_struct_defs.keys()
    }

    pub fn add_named_struct_def(&mut self, name: String, def: NamedStructDef) {
        match self.named_struct_defs.entry(name) {
            Entry::Occupied(_) => {
                panic!("Trying to redefine named struct");
            },
            Entry::Vacant(ventry) => {
                ventry.insert(def);
            },
        }
    }

    pub fn remove_named_struct_def(&mut self, name: &str) -> bool {
        self.named_struct_defs.remove(name).is_some()
    }

    pub fn get_for_type(&self, ty: &InstType) -> TypeRef {
        match ty {
            InstType::VoidType => self.void(),
            InstType::IntegerType { bits } => self.int(*bits),
            InstType::PointerType { addr_space } => self.pointer_in_addr_space(*addr_space),
            InstType::FPType(fpt) => self.fp(*fpt),
            InstType::FuncType { result_type, param_types } => self.func_type(result_type.clone(), param_types.clone()),
            InstType::VectorType { element_type, num_elements } => self.vector_of(element_type.clone(), *num_elements),
            InstType::ArrayType { element_type, num_elements } => self.array_of(element_type.clone(), *num_elements),
            InstType::StructType { element_types } => self.struct_of(element_types.clone()),
            InstType::NamedStructType { name } => self.named_struct(name),
        }
    }
}

#[derive(Clone, Debug)]
struct TypeCache<K: Eq + Hash + Clone> {
    map: HashMap<K, TypeRef>,
}

#[allow(dead_code)]
impl<K: Eq + Hash + Clone> TypeCache<K> {
    fn new() -> Self {
        Self { map: HashMap::new() }
    }

    fn lookup<Q: ?Sized>(&self, key: &Q) -> Option<TypeRef>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map.get(key).cloned()
    }

    fn lookup_or_insert(&mut self, key: K, if_missing: impl FnOnce() -> InstType) -> TypeRef {
        self.map
            .entry(key)
            .or_insert_with(|| TypeRef::new(if_missing()))
            .clone()
    }

    fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
}
