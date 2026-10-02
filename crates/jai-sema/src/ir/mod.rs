//! Checked semantic storage, expressions, and control flow.
mod control;
mod expressions;
mod storage;
pub use control::*;
pub use expressions::*;
use jai_types::{TypeId, Types};
pub use storage::*;

macro_rules! id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name(pub(crate) usize);
        impl $name {
            pub fn index(self) -> usize {
                self.0
            }
        }
    };
}
id!(ProcedureId);
id!(ParameterId);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalId {
    pub(crate) procedure: ProcedureId,
    pub(crate) index: usize,
}
impl LocalId {
    pub fn index(self) -> usize {
        self.index
    }
    pub fn procedure(self) -> ProcedureId {
        self.procedure
    }
}
id!(GlobalId);
id!(LoopId);
id!(CleanupId);

#[derive(Clone, Copy, Debug)]
pub enum EntryPoint {
    Void(ProcedureId),
    Int(ProcedureId),
}
#[derive(Debug)]
pub struct Program {
    pub(crate) procedures: Vec<Procedure>,
    pub(crate) entry: EntryPoint,
    pub(crate) globals: Vec<Global>,
    pub(crate) types: Types,
}
impl Program {
    pub fn procedures(&self) -> &[Procedure] {
        &self.procedures
    }
    pub fn globals(&self) -> &[Global] {
        &self.globals
    }
    pub fn entry(&self) -> EntryPoint {
        self.entry
    }
    pub fn types(&self) -> &Types {
        &self.types
    }
}
#[derive(Debug)]
pub struct Procedure {
    pub id: ProcedureId,
    pub signature: TypeId,
    pub parameters: Vec<Local>,
    pub locals: Vec<Local>,
    pub body: Block,
    pub cleanups: Vec<Block>,
}

#[cfg(test)]
mod tests {
    use jai_types::{IntegerType, TypeKind};
    #[test]
    fn resolved_scalar_storage_and_signatures_share_one_registry() {
        let module = jai_syntax::parse("enabled: bool = true; take :: (x: u8) -> u16 { return x; } main :: () -> int { n:u8=7; return take(n); }").unwrap();
        let program = crate::resolve(&module).unwrap();
        let types = program.types();
        for procedure in program.procedures() {
            let TypeKind::Procedure(id) = types.kind(procedure.signature).unwrap() else {
                panic!("procedure signature kind");
            };
            let signature = types.procedure(*id).unwrap();
            assert_eq!(signature.parameters.len(), procedure.parameters.len());
            for (parameter, &ty) in procedure.parameters.iter().zip(&signature.parameters) {
                assert_eq!(parameter.ty(), ty);
                assert_eq!(parameter.id().procedure(), procedure.id);
                assert_eq!(parameter.place().ty(), ty);
            }
            for local in &procedure.locals {
                assert_eq!(local.id().procedure(), procedure.id);
                assert!(matches!(
                    types.kind(local.ty()),
                    Ok(TypeKind::Bool | TypeKind::Integer(_))
                ));
            }
        }
        let global = program.globals()[0];
        assert!(matches!(types.kind(global.ty()), Ok(TypeKind::Bool)));
        assert_eq!(global.place().ty(), global.ty());
        let parameter = program.procedures()[0].parameters[0];
        assert!(matches!(
            types.kind(parameter.ty()),
            Ok(TypeKind::Integer(IntegerType::U8))
        ));
    }
}
