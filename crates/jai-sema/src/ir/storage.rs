use super::{GlobalId, LocalId, ProcedureId};
use jai_types::Integer;
use jai_types::{IntegerType, ScalarType, TypeId, TypeKind, TypeRegistry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceKind {
    Local(LocalId),
    Global(GlobalId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    kind: PlaceKind,
    ty: TypeId,
}
impl Place {
    pub fn kind(self) -> PlaceKind {
        self.kind
    }
    pub fn ty(self) -> TypeId {
        self.ty
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Local {
    id: LocalId,
    ty: TypeId,
}
impl Local {
    pub(crate) fn new(
        procedure: ProcedureId,
        index: usize,
        domain: ScalarType,
        types: &TypeRegistry,
    ) -> Self {
        Self {
            id: LocalId { procedure, index },
            ty: types.scalar(domain),
        }
    }
    pub fn id(self) -> LocalId {
        self.id
    }
    pub fn ty(self) -> TypeId {
        self.ty
    }
    pub fn place(self) -> Place {
        Place {
            kind: PlaceKind::Local(self.id),
            ty: self.ty,
        }
    }
    pub(crate) fn integer(self, types: &TypeRegistry) -> Option<IntLocal> {
        match types.kind(self.ty).ok()? {
            TypeKind::Integer(ty) => Some(IntLocal {
                local: self,
                ty: *ty,
            }),
            _ => None,
        }
    }
    pub(crate) fn boolean(self, types: &TypeRegistry) -> Option<BoolLocal> {
        matches!(types.kind(self.ty), Ok(TypeKind::Bool)).then_some(BoolLocal(self))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct IntLocal {
    local: Local,
    ty: IntegerType,
}
impl IntLocal {
    pub fn index(self) -> usize {
        self.local.id.index()
    }
    pub fn ty(self) -> IntegerType {
        self.ty
    }
    pub fn local(self) -> Local {
        self.local
    }
    pub fn place(self) -> IntPlace {
        IntPlace {
            place: self.local.place(),
            ty: self.ty,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct BoolLocal(Local);
impl BoolLocal {
    pub fn index(self) -> usize {
        self.0.id.index()
    }
    pub fn local(self) -> Local {
        self.0
    }
    pub fn place(self) -> BoolPlace {
        BoolPlace(self.0.place())
    }
}
#[derive(Clone, Copy, Debug)]
pub struct IntPlace {
    place: Place,
    ty: IntegerType,
}
impl IntPlace {
    pub fn ty(self) -> IntegerType {
        self.ty
    }
    pub fn place(self) -> Place {
        self.place
    }
}
#[derive(Clone, Copy, Debug)]
pub struct BoolPlace(Place);
impl BoolPlace {
    pub fn place(self) -> Place {
        self.0
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Storage {
    Int(IntPlace),
    Bool(BoolPlace),
}
impl Storage {
    pub fn place(self) -> Place {
        match self {
            Self::Int(p) => p.place(),
            Self::Bool(p) => p.place(),
        }
    }
}
impl Storage {
    pub(crate) fn local(local: Local, types: &TypeRegistry) -> Self {
        if let Some(integer) = local.integer(types) {
            Self::Int(integer.place())
        } else {
            Self::Bool(local.boolean(types).expect("scalar local storage").place())
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum GlobalInitializer {
    Int(Integer),
    Bool(bool),
}
#[derive(Clone, Copy, Debug)]
pub struct Global {
    id: GlobalId,
    ty: TypeId,
    initializer: GlobalInitializer,
}
impl Global {
    pub(crate) fn new(index: usize, initializer: GlobalInitializer, types: &TypeRegistry) -> Self {
        let domain = match initializer {
            GlobalInitializer::Int(n) => ScalarType::Int(n.ty()),
            GlobalInitializer::Bool(_) => ScalarType::Bool,
        };
        Self {
            id: GlobalId(index),
            ty: types.scalar(domain),
            initializer,
        }
    }
    pub fn id(self) -> GlobalId {
        self.id
    }
    pub fn ty(self) -> TypeId {
        self.ty
    }
    pub fn initializer(self) -> GlobalInitializer {
        self.initializer
    }
    pub fn place(self) -> Place {
        Place {
            kind: PlaceKind::Global(self.id),
            ty: self.ty,
        }
    }
    pub(crate) fn storage(self) -> Storage {
        match self.initializer {
            GlobalInitializer::Int(value) => Storage::Int(IntPlace {
                place: self.place(),
                ty: value.ty(),
            }),
            GlobalInitializer::Bool(_) => Storage::Bool(BoolPlace(self.place())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scalar_views_use_the_owning_registry_and_preserve_common_roots() {
        let types = TypeRegistry::new();
        let foreign = TypeRegistry::new();
        let local = Local::new(ProcedureId(2), 3, ScalarType::Int(IntegerType::U8), &types);
        let integer = local.integer(&types).unwrap();
        assert_eq!(integer.place().place(), local.place());
        assert_eq!(integer.local().id().procedure(), ProcedureId(2));
        assert!(local.boolean(&types).is_none());
        assert!(local.integer(&foreign).is_none());
        let local_bool = Local::new(ProcedureId(2), 4, ScalarType::Bool, &types);
        assert_eq!(
            local_bool.boolean(&types).unwrap().place().place(),
            local_bool.place()
        );
        assert!(local_bool.integer(&types).is_none());
        let global = Global::new(0, GlobalInitializer::Bool(true), &types);
        assert_eq!(global.storage().place(), global.place());
        let frozen = types.freeze().unwrap();
        assert!(matches!(
            frozen.kind(local.ty()),
            Ok(TypeKind::Integer(IntegerType::U8))
        ));
        assert!(matches!(frozen.kind(global.ty()), Ok(TypeKind::Bool)));
    }
}
