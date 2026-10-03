//! Borrow source policy allocations without cloning or resolving their syntax.
use crate::*;
use std::mem::size_of;

mod expressions;
mod files;
mod records;
mod statements;
#[cfg(test)]
mod tests;

pub const MAX_SOURCE_METADATA_DEPTH: usize = 128;

#[derive(Debug, PartialEq, Eq)]
pub enum SourceMetadataError<E> {
    Admission(E),
    ExcessiveNesting,
}

type Result<E> = std::result::Result<(), SourceMetadataError<E>>;

struct Visitor<'a, A> {
    admit: &'a mut A,
}

impl<A> Visitor<'_, A> {
    fn admit<E>(&mut self, work: usize, bytes: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        (self.admit)(work, bytes).map_err(SourceMetadataError::Admission)
    }

    fn node<E>(&mut self, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        if depth > MAX_SOURCE_METADATA_DEPTH {
            return Err(SourceMetadataError::ExcessiveNesting);
        }
        self.admit(1, 0)
    }

    fn allocation<T, E>(&mut self, capacity: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.admit(
            capacity.saturating_add(1),
            capacity.saturating_mul(size_of::<T>()),
        )
    }

    fn sequence<T, E>(
        &mut self,
        items: &Vec<T>,
        depth: usize,
        mut visit: impl FnMut(&mut Self, &T, usize) -> Result<E>,
    ) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.allocation::<T, E>(items.capacity())?;
        for item in items {
            visit(self, item, depth + 1)?;
        }
        Ok(())
    }

    fn boxed<T, E>(
        &mut self,
        item: &T,
        depth: usize,
        visit: impl FnOnce(&mut Self, &T, usize) -> Result<E>,
    ) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.allocation::<T, E>(1)?;
        visit(self, item, depth + 1)
    }

    fn path<E>(&mut self, path: &NamePath, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.allocation::<Symbol, E>(path.members.capacity())
    }

    fn text<E>(&mut self, text: &String) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.allocation::<u8, E>(text.capacity())
    }

    fn procedure_type<E>(&mut self, source: &ProcedureTypeSyntax, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.sequence(&source.parameters, depth, Self::type_parameter)?;
        self.sequence(&source.results, depth, Self::type_parameter)
    }

    fn type_parameter<E>(&mut self, parameter: &ProcedureTypeParameter, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.ty(&parameter.ty, depth + 1)?;
        if let Some(default) = &parameter.default {
            self.expression(default, depth + 1)?;
        }
        Ok(())
    }

    fn ty<E>(&mut self, ty: &TypeSyntax, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match ty {
            TypeSyntax::This | TypeSyntax::Builtin(_) | TypeSyntax::Variable(_) => Ok(()),
            TypeSyntax::Named(path) => self.path(path, depth + 1),
            TypeSyntax::TypeOf(value) => self.boxed(value.as_ref(), depth, Self::expression),
            TypeSyntax::Pointer(ty)
            | TypeSyntax::Slice(ty)
            | TypeSyntax::DynamicArray(ty)
            | TypeSyntax::Variant {
                base: ty, ..
            } => self.boxed(ty.as_ref(), depth, Self::ty),
            TypeSyntax::Restricted {
                restriction, ..
            } => match restriction {
                TypeRestrictionSyntax::Nominal(ty) | TypeRestrictionSyntax::Interface(ty) => {
                    self.boxed(ty.as_ref(), depth, Self::ty)
                }
            },
            TypeSyntax::FixedArray {
                count,
                element,
            } => {
                self.boxed(count.as_ref(), depth, Self::expression)?;
                self.boxed(element.as_ref(), depth, Self::ty)
            }
            TypeSyntax::Procedure(source) => self.procedure_type(source, depth + 1),
            TypeSyntax::Application(source) => {
                self.boxed(source.base.as_ref(), depth, Self::ty)?;
                self.sequence(&source.arguments, depth, Self::argument)
            }
            TypeSyntax::InlineRecord(source) => {
                self.boxed(source.as_ref(), depth, Self::inline_record)
            }
            TypeSyntax::InlineEnum(source) => self.boxed(source.as_ref(), depth, Self::inline_enum),
        }
    }
}

impl ProcedureTypeSyntax {
    /// Visit the heap storage owned by this original annotation. Its inline
    /// struct is counted by its containing source receipt, not counted again.
    /// Admission precedes descent into each node and allocated container.
    pub fn visit_retained_metadata<E>(
        &self,
        admit: &mut impl FnMut(usize, usize) -> std::result::Result<(), E>,
    ) -> std::result::Result<(), SourceMetadataError<E>> {
        Visitor {
            admit,
        }
        .procedure_type(self, 0)
    }
}
