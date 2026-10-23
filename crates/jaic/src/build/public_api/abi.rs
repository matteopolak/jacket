//! Resolved declaration layouts, not offsets assumed from a particular stdlib.
use super::Operation;
use crate::{
    fxhash::HashMap,
    interp::{Interp, Trap},
    ir,
    types::TypeId,
};
use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct Field {
    pub name: String,
    pub offset: u64,
    pub ty: TypeId,
    pub promoted: bool,
}

#[derive(Clone)]
pub(crate) enum Kind {
    Scalar,
    String,
    Pointer(TypeId),
    Array {
        element: TypeId,
        count: Option<u64>,
        resizable: bool,
    },
    Record(Vec<Field>),
}

#[derive(Clone)]
pub(crate) struct Layout {
    pub size: u64,
    pub align: u64,
    pub name: String,
    pub kind: Kind,
    pub descriptor: Option<ir::GlobalId>,
}

#[derive(Clone, Default)]
pub(crate) struct Schema {
    pub types: HashMap<TypeId, Layout>,
    pub enums: HashMap<TypeId, Vec<(String, i64)>>,
    /// The records the owning modules declare, by name.
    pub names: HashMap<String, TypeId>,
}

#[derive(Clone)]
pub(crate) struct Binding {
    pub operation: Operation,
    pub parameters: Vec<TypeId>,
    pub returns: Vec<(TypeId, bool)>,
    pub context: bool,
    pub schema: Rc<Schema>,
    /// The build options record and its defaults, for the operations that fill one.
    pub options: Option<(TypeId, ir::GlobalId)>,
}

impl Schema {
    pub fn get(&self, ty: TypeId) -> &Layout {
        &self.types[&ty]
    }

    pub fn field(&self, ty: TypeId, name: &str) -> Result<(u64, TypeId), Trap> {
        let Kind::Record(fields) = &self.get(ty).kind else {
            return Err(Trap::new(format!(
                "compiler API expected a record, got {}",
                self.get(ty).name
            )));
        };
        if let Some(field) = fields.iter().find(|field| field.name == name) {
            return Ok((field.offset, field.ty));
        }
        for field in fields.iter().filter(|field| field.promoted) {
            if let Ok((offset, ty)) = self.field(field.ty, name) {
                return Ok((field.offset + offset, ty));
            }
        }
        Err(Trap::new(format!(
            "compiler API record {} has no field {name}",
            self.get(ty).name
        )))
    }

    pub fn named(&self, name: &str) -> Result<TypeId, Trap> {
        self.names
            .get(name)
            .copied()
            .ok_or_else(|| Trap::new(format!("compiler API needs the {name} record")))
    }

    pub fn pointed(&self, ty: TypeId) -> Result<TypeId, Trap> {
        match self.get(ty).kind {
            Kind::Pointer(inner) => Ok(inner),
            _ => Err(Trap::new("compiler API expected a pointer")),
        }
    }

    pub fn read_number(&self, interp: &Interp, ty: TypeId, address: u64) -> Result<u64, Trap> {
        let size = self.get(ty).size as usize;
        if size > 8 {
            return Err(Trap::new("compiler API expected a scalar"));
        }
        let bytes = interp
            .read(address, size)
            .ok_or_else(|| Trap::unreadable(address, size))?;
        let mut value = [0; 8];
        value[..size].copy_from_slice(&bytes);
        Ok(u64::from_le_bytes(value))
    }

    pub fn write_number(
        &self,
        interp: &mut Interp,
        ty: TypeId,
        address: u64,
        value: u64,
    ) -> Result<(), Trap> {
        let size = self.get(ty).size as usize;
        if size > 8 {
            return Err(Trap::new("compiler API expected a scalar"));
        }
        write(interp, address, &value.to_le_bytes()[..size])
    }
}

pub(super) fn write(interp: &mut Interp, address: u64, bytes: &[u8]) -> Result<(), Trap> {
    if interp.write(address, bytes) {
        Ok(())
    } else {
        Err(Trap::unwritable(address, bytes.len()))
    }
}

pub(super) fn bytes(interp: &Interp, address: u64, size: usize) -> Result<Vec<u8>, Trap> {
    interp
        .read(address, size)
        .ok_or_else(|| Trap::unreadable(address, size))
}

pub(super) fn view(interp: &Interp, address: u64) -> Result<(u64, u64), Trap> {
    interp
        .read_pair(address)
        .ok_or_else(|| Trap::unreadable(address, 16))
}

pub(super) fn string(interp: &Interp, address: u64) -> Result<String, Trap> {
    let (count, data) = view(interp, address)?;
    Ok(String::from_utf8_lossy(&bytes(
        interp,
        data,
        usize::try_from(count).map_err(|_| Trap::new("invalid string size"))?,
    )?)
    .into_owned())
}

pub(super) fn put_view(
    interp: &mut Interp,
    address: u64,
    count: u64,
    data: u64,
) -> Result<(), Trap> {
    write(interp, address, &count.to_le_bytes())?;
    write(interp, address + 8, &data.to_le_bytes())
}

pub(super) fn put_string(interp: &mut Interp, address: u64, text: &[u8]) -> Result<(), Trap> {
    let data = interp.register_data_global(text, 1, false, 1)?;
    put_view(interp, address, text.len() as u64, data)
}
