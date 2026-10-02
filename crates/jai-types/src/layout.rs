//! Explicit target layouts, independent of Rust's host representation.
use crate::{FloatType, IntegerType, RecordKind, TypeError, TypeId, TypeKind, Types};
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScalarLayout {
    pub size: u64,
    pub alignment: u32,
}
impl ScalarLayout {
    pub const fn new(size: u64, alignment: u32) -> Self {
        Self { size, alignment }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyIssue {
    ZeroSize,
    AlignmentNotPowerOfTwo,
    SizeNotAligned,
    IncorrectWidth,
}
#[derive(Debug)]
pub enum LayoutError {
    InvalidPolicy {
        component: &'static str,
        issue: PolicyIssue,
    },
    Type(TypeError),
    Unsized(TypeId),
    RecursiveValue(TypeId),
    Overflow(TypeId),
    DependencyNotReady(TypeId),
}
impl From<TypeError> for LayoutError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy { component, issue } => {
                write!(f, "invalid layout policy for {component}: {issue:?}")
            }
            Self::Type(error) => write!(f, "invalid type for layout: {error:?}"),
            Self::Unsized(id) => write!(f, "type {id:?} has no storage layout"),
            Self::RecursiveValue(id) => write!(f, "recursive value layout for {id:?}"),
            Self::Overflow(id) => write!(f, "layout size overflows for {id:?}"),
            Self::DependencyNotReady(id) => write!(f, "layout dependency unavailable for {id:?}"),
        }
    }
}
impl std::error::Error for LayoutError {}

/// Explicit primitive storage policy; procedure values use pointer layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutPolicy {
    pointer: ScalarLayout,
    integers: [ScalarLayout; 4],
    floats: [ScalarLayout; 2],
    boolean: ScalarLayout,
}
impl LayoutPolicy {
    pub fn new(
        pointer: ScalarLayout,
        integers: [ScalarLayout; 4],
        floats: [ScalarLayout; 2],
        boolean: ScalarLayout,
    ) -> Result<Self, LayoutError> {
        fn validate(
            component: &'static str,
            layout: ScalarLayout,
            width: Option<u64>,
        ) -> Result<(), LayoutError> {
            let issue = if layout.size == 0 {
                Some(PolicyIssue::ZeroSize)
            } else if !layout.alignment.is_power_of_two() {
                Some(PolicyIssue::AlignmentNotPowerOfTwo)
            } else if !layout.size.is_multiple_of(u64::from(layout.alignment)) {
                Some(PolicyIssue::SizeNotAligned)
            } else if width.is_some_and(|width| layout.size != width) {
                Some(PolicyIssue::IncorrectWidth)
            } else {
                None
            };
            if let Some(issue) = issue {
                Err(LayoutError::InvalidPolicy { component, issue })
            } else {
                Ok(())
            }
        }
        validate("pointer", pointer, None)?;
        for (i, layout) in integers.iter().enumerate() {
            validate("integer", *layout, Some(1 << i))?;
        }
        for (i, layout) in floats.iter().enumerate() {
            validate("float", *layout, Some(4 << i))?;
        }
        validate("bool", boolean, Some(1))?;
        Ok(Self {
            pointer,
            integers,
            floats,
            boolean,
        })
    }
    /// Conventional LP64 profile, explicitly selected by the caller.
    pub fn lp64() -> Self {
        Self {
            pointer: ScalarLayout::new(8, 8),
            integers: [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 8),
            ],
            floats: [ScalarLayout::new(4, 4), ScalarLayout::new(8, 8)],
            boolean: ScalarLayout::new(1, 1),
        }
    }
    pub fn pointer(self) -> ScalarLayout {
        self.pointer
    }
    pub fn integer(self, ty: IntegerType) -> ScalarLayout {
        self.integers[match ty {
            IntegerType::S8 | IntegerType::U8 => 0,
            IntegerType::S16 | IntegerType::U16 => 1,
            IntegerType::S32 | IntegerType::U32 => 2,
            IntegerType::S64 | IntegerType::U64 => 3,
        }]
    }
    pub fn float(self, ty: FloatType) -> ScalarLayout {
        self.floats[match ty {
            FloatType::F32 => 0,
            FloatType::F64 => 1,
        }]
    }
    pub fn boolean(self) -> ScalarLayout {
        self.boolean
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    pub size: u64,
    pub alignment: u32,
    /// Record or descriptor fields in declaration order; unions use zero offsets.
    pub field_offsets: Box<[u64]>,
    pub array_stride: Option<u64>,
}
impl From<ScalarLayout> for Layout {
    fn from(value: ScalarLayout) -> Self {
        Self {
            size: value.size,
            alignment: value.alignment,
            field_offsets: Box::new([]),
            array_stride: None,
        }
    }
}

pub struct LayoutEngine<'types> {
    types: &'types Types,
    policy: LayoutPolicy,
    cache: HashMap<TypeId, Layout>,
}
impl<'types> LayoutEngine<'types> {
    pub fn new(types: &'types Types, policy: LayoutPolicy) -> Self {
        Self {
            types,
            policy,
            cache: HashMap::new(),
        }
    }
    pub fn policy(&self) -> LayoutPolicy {
        self.policy
    }
    /// Iterative postorder traversal avoids recursion on deeply nested records.
    pub fn layout(&mut self, root: TypeId) -> Result<&Layout, LayoutError> {
        let mut stack = vec![(root, false)];
        let mut visiting = HashSet::new();
        while let Some((id, finish)) = stack.pop() {
            if self.cache.contains_key(&id) {
                continue;
            }
            if finish {
                let layout = self.calculate(id)?;
                self.cache.insert(id, layout);
                visiting.remove(&id);
                continue;
            }
            if !visiting.insert(id) {
                return Err(LayoutError::RecursiveValue(id));
            }
            stack.push((id, true));
            match self.types.kind(id)? {
                TypeKind::Record(record) => {
                    for field in self.types.record(*record)?.fields.iter().rev() {
                        stack.push((*field, false));
                    }
                }
                TypeKind::FixedArray { element, .. } => stack.push((*element, false)),
                // Pointer and descriptor representations do not need pointee layout.
                _ => {}
            }
        }
        self.cache
            .get(&root)
            .ok_or(LayoutError::DependencyNotReady(root))
    }
    fn dependency(&self, id: TypeId) -> Result<&Layout, LayoutError> {
        self.cache
            .get(&id)
            .ok_or(LayoutError::DependencyNotReady(id))
    }
    fn align_up(id: TypeId, size: u64, alignment: u32) -> Result<u64, LayoutError> {
        let mask = u64::from(alignment) - 1;
        size.checked_add(mask)
            .map(|size| size & !mask)
            .ok_or(LayoutError::Overflow(id))
    }
    fn fields<'a>(
        id: TypeId,
        fields: impl IntoIterator<Item = &'a Layout>,
        union: bool,
    ) -> Result<Layout, LayoutError> {
        let mut size = 0;
        let mut alignment = 1;
        let mut offsets = Vec::new();
        for field in fields {
            alignment = alignment.max(field.alignment);
            if union {
                offsets.push(0);
                size = size.max(field.size);
            } else {
                size = Self::align_up(id, size, field.alignment)?;
                offsets.push(size);
                size = size
                    .checked_add(field.size)
                    .ok_or(LayoutError::Overflow(id))?;
            }
        }
        Ok(Layout {
            size: Self::align_up(id, size, alignment)?,
            alignment,
            field_offsets: offsets.into_boxed_slice(),
            array_stride: None,
        })
    }
    fn calculate(&self, id: TypeId) -> Result<Layout, LayoutError> {
        Ok(match self.types.kind(id)? {
            TypeKind::Void | TypeKind::Type => return Err(LayoutError::Unsized(id)),
            TypeKind::Bool => self.policy.boolean.into(),
            TypeKind::Integer(ty) => self.policy.integer(*ty).into(),
            TypeKind::Float(ty) => self.policy.float(*ty).into(),
            TypeKind::Pointer(_) | TypeKind::Procedure(_) => self.policy.pointer.into(),
            TypeKind::Enum(enumeration) => self
                .policy
                .integer(self.types.enumeration(*enumeration)?.representation)
                .into(),
            TypeKind::FixedArray { element, count } => {
                let element = self.dependency(*element)?;
                let stride = Self::align_up(id, element.size, element.alignment)?;
                Layout {
                    size: stride
                        .checked_mul(*count)
                        .ok_or(LayoutError::Overflow(id))?,
                    alignment: element.alignment,
                    field_offsets: Box::new([]),
                    array_stride: Some(stride),
                }
            }
            TypeKind::Record(record) => {
                let record = self.types.record(*record)?;
                let fields = record
                    .fields
                    .iter()
                    .map(|field| self.dependency(*field))
                    .collect::<Result<Vec<_>, _>>()?;
                Self::fields(id, fields, record.kind == RecordKind::Union)?
            }
            TypeKind::String | TypeKind::Slice(_) => {
                let count: Layout = self.policy.integer(IntegerType::S64).into();
                let pointer: Layout = self.policy.pointer.into();
                Self::fields(id, [&count, &pointer], false)?
            }
            TypeKind::DynamicArray(_) => {
                let count: Layout = self.policy.integer(IntegerType::S64).into();
                let pointer: Layout = self.policy.pointer.into();
                let allocator = Self::fields(id, [&pointer, &pointer], false)?;
                Self::fields(id, [&count, &pointer, &count, &allocator], false)?
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScalarType, TypeRegistry};
    fn ilp32() -> LayoutPolicy {
        LayoutPolicy::new(
            ScalarLayout::new(4, 4),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 4),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
            ScalarLayout::new(1, 1),
        )
        .unwrap()
    }
    #[test]
    fn descriptor_layouts_use_explicit_target_alignment() {
        let mut registry = TypeRegistry::new();
        let element = registry.scalar(ScalarType::Int(IntegerType::U8));
        let slice = registry.slice(element).unwrap();
        let dynamic = registry.dynamic_array(element).unwrap();
        let string = registry.string();
        let types = registry.freeze().unwrap();
        let mut wide = LayoutEngine::new(&types, LayoutPolicy::lp64());
        assert_eq!(
            wide.layout(slice).unwrap(),
            &Layout {
                size: 16,
                alignment: 8,
                field_offsets: vec![0, 8].into_boxed_slice(),
                array_stride: None
            }
        );
        assert_eq!(
            wide.layout(dynamic).unwrap(),
            &Layout {
                size: 40,
                alignment: 8,
                field_offsets: vec![0, 8, 16, 24].into_boxed_slice(),
                array_stride: None
            }
        );
        let string_layout = wide.layout(string).unwrap().clone();
        assert_eq!(&string_layout, wide.layout(slice).unwrap());
        let mut narrow = LayoutEngine::new(&types, ilp32());
        assert_eq!(
            narrow.layout(slice).unwrap(),
            &Layout {
                size: 12,
                alignment: 4,
                field_offsets: vec![0, 8].into_boxed_slice(),
                array_stride: None
            }
        );
        assert_eq!(
            narrow.layout(dynamic).unwrap(),
            &Layout {
                size: 28,
                alignment: 4,
                field_offsets: vec![0, 8, 12, 20].into_boxed_slice(),
                array_stride: None
            }
        );
    }
    #[test]
    fn records_unions_arrays_and_tail_padding_keep_declaration_order() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let word = registry.scalar(ScalarType::Int(IntegerType::U32));
        let inner = registry.reserve_record(RecordKind::Struct);
        registry.define_record(inner, vec![byte, word]).unwrap();
        let outer = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record(outer, vec![byte, inner, byte])
            .unwrap();
        let union = registry.reserve_record(RecordKind::Union);
        registry.define_record(union, vec![byte, outer]).unwrap();
        let array = registry.fixed_array(inner, 3).unwrap();
        let empty = registry.fixed_array(inner, 0).unwrap();
        let types = registry.freeze().unwrap();
        let mut engine = LayoutEngine::new(&types, LayoutPolicy::lp64());
        assert_eq!(
            engine.layout(inner).unwrap().field_offsets.as_ref(),
            &[0, 4]
        );
        assert_eq!(
            engine.layout(outer).unwrap(),
            &Layout {
                size: 16,
                alignment: 4,
                field_offsets: vec![0, 4, 12].into_boxed_slice(),
                array_stride: None
            }
        );
        assert_eq!(
            engine.layout(union).unwrap(),
            &Layout {
                size: 16,
                alignment: 4,
                field_offsets: vec![0, 0].into_boxed_slice(),
                array_stride: None
            }
        );
        assert_eq!(engine.layout(array).unwrap().size, 24);
        assert_eq!(engine.layout(array).unwrap().array_stride, Some(8));
        assert_eq!(engine.layout(empty).unwrap().size, 0);
        assert_eq!(engine.layout(empty).unwrap().alignment, 4);
    }
    #[test]
    fn recursive_pointer_records_have_finite_layouts() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let record = registry.reserve_record(RecordKind::Struct);
        let pointer = registry.pointer(record).unwrap();
        registry.define_record(record, vec![byte, pointer]).unwrap();
        let types = registry.freeze().unwrap();
        let mut engine = LayoutEngine::new(&types, LayoutPolicy::lp64());
        assert_eq!(engine.layout(record).unwrap().size, 16);
        assert_eq!(
            engine.layout(record).unwrap().field_offsets.as_ref(),
            &[0, 8]
        );
    }
    #[test]
    fn arithmetic_overflow_and_void_storage_are_structured_errors() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let word = registry.scalar(ScalarType::Int(IntegerType::U64));
        let huge = registry.fixed_array(byte, u64::MAX).unwrap();
        let multiply_overflow = registry.fixed_array(word, u64::MAX).unwrap();
        let add_overflow = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record(add_overflow, vec![huge, byte])
            .unwrap();
        let align_overflow = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record(align_overflow, vec![huge, word])
            .unwrap();
        let void = registry.void();
        let types = registry.freeze().unwrap();
        let mut engine = LayoutEngine::new(&types, LayoutPolicy::lp64());
        assert!(
            matches!(engine.layout(multiply_overflow), Err(LayoutError::Overflow(id)) if id == multiply_overflow)
        );
        assert!(
            matches!(engine.layout(add_overflow), Err(LayoutError::Overflow(id)) if id == add_overflow)
        );
        assert!(
            matches!(engine.layout(align_overflow), Err(LayoutError::Overflow(id)) if id == align_overflow)
        );
        assert!(matches!(engine.layout(void), Err(LayoutError::Unsized(id)) if id == void));
        assert_eq!(engine.layout(byte).unwrap().size, 1); // Failed queries do not poison the cache.
    }
    #[test]
    fn policy_rejects_invalid_alignment_and_semantic_widths() {
        let valid = LayoutPolicy::lp64();
        for pointer in [
            ScalarLayout::new(0, 1),
            ScalarLayout::new(8, 0),
            ScalarLayout::new(8, 3),
            ScalarLayout::new(4, 8),
        ] {
            assert!(matches!(
                LayoutPolicy::new(pointer, valid.integers, valid.floats, valid.boolean),
                Err(LayoutError::InvalidPolicy {
                    component: "pointer",
                    ..
                })
            ));
        }
        let mut integers = valid.integers;
        integers[0] = ScalarLayout::new(2, 1);
        assert!(matches!(
            LayoutPolicy::new(valid.pointer, integers, valid.floats, valid.boolean),
            Err(LayoutError::InvalidPolicy {
                issue: PolicyIssue::IncorrectWidth,
                ..
            })
        ));
    }
    #[test]
    fn deeply_nested_records_use_an_iterative_layout_walk() {
        let mut registry = TypeRegistry::new();
        let mut root = registry.scalar(ScalarType::Int(IntegerType::U64));
        for _ in 0..4096 {
            let record = registry.reserve_record(RecordKind::Struct);
            registry.define_record(record, vec![root]).unwrap();
            root = record;
        }
        let types = registry.freeze().unwrap();
        let mut engine = LayoutEngine::new(&types, LayoutPolicy::lp64());
        assert_eq!(engine.layout(root).unwrap().size, 8);
    }

    #[test]
    fn shared_dependencies_and_zero_size_fields_keep_alignment() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let word = registry.scalar(ScalarType::Int(IntegerType::U64));
        let shared = registry.reserve_record(RecordKind::Struct);
        registry.define_record(shared, [word, byte]).unwrap();
        let empty = registry.reserve_record(RecordKind::Struct);
        registry.define_record(empty, []).unwrap();
        let empty_array = registry.fixed_array(shared, 0).unwrap();
        let left = registry.reserve_record(RecordKind::Struct);
        registry.define_record(left, [shared, empty_array]).unwrap();
        let right = registry.reserve_record(RecordKind::Struct);
        registry.define_record(right, [shared, byte]).unwrap();
        let root = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record(root, [left, right, shared, shared, empty])
            .unwrap();
        let aligned_empty = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record(aligned_empty, [byte, empty_array, byte])
            .unwrap();
        let types = registry.freeze().unwrap();
        let mut engine = LayoutEngine::new(&types, LayoutPolicy::lp64());
        assert_eq!(
            engine.layout(root).unwrap(),
            &Layout {
                size: 72,
                alignment: 8,
                field_offsets: Box::new([0, 16, 40, 56, 72]),
                array_stride: None,
            }
        );
        assert_eq!(
            engine.layout(aligned_empty).unwrap(),
            &Layout {
                size: 16,
                alignment: 8,
                field_offsets: Box::new([0, 8, 8]),
                array_stride: None,
            }
        );
        assert_eq!(
            engine.layout(empty).unwrap(),
            &Layout {
                size: 0,
                alignment: 1,
                field_offsets: Box::new([]),
                array_stride: None,
            }
        );
        // Reusing the root must preserve the result after querying another branch.
        assert_eq!(engine.layout(root).unwrap().size, 72);
    }
}

#[cfg(test)]
mod representation_tests {
    use super::*;
    use crate::{CallingConvention, ContextMode, Integer, ProcedureType, ScalarType, TypeRegistry};
    #[test]
    fn enum_aliases_float_fields_and_procedure_values_share_registry_layouts() {
        let mut registry = TypeRegistry::new();
        let enumeration = registry.reserve_enum(IntegerType::U16);
        let value = Integer::checked(IntegerType::U16, 7).unwrap();
        registry
            .define_enum(enumeration, vec![value, value])
            .unwrap();
        let double = registry.float(FloatType::F64);
        let boolean = registry.scalar(ScalarType::Bool);
        let procedure = registry
            .procedure(ProcedureType {
                parameters: vec![enumeration].into_boxed_slice(),
                results: vec![double].into_boxed_slice(),
                convention: CallingConvention::Jai,
                context: ContextMode::None,
            })
            .unwrap();
        let record = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record(record, vec![boolean, double, procedure, enumeration])
            .unwrap();
        let meta_type = registry.meta_type();
        let types = registry.freeze().unwrap();
        let mut engine = LayoutEngine::new(&types, LayoutPolicy::lp64());
        assert_eq!(engine.layout(enumeration).unwrap().size, 2);
        assert_eq!(
            engine.layout(record).unwrap().field_offsets.as_ref(),
            &[0, 8, 16, 24]
        );
        assert_eq!(engine.layout(record).unwrap().size, 32);
        assert!(
            matches!(engine.layout(meta_type), Err(LayoutError::Unsized(id)) if id == meta_type)
        );
    }
    #[test]
    fn ids_from_another_registry_are_rejected_before_cache_access() {
        let registry = TypeRegistry::new();
        let foreign = TypeRegistry::new();
        let foreign_id = foreign.scalar(ScalarType::Bool);
        let types = registry.freeze().unwrap();
        let mut engine = LayoutEngine::new(&types, LayoutPolicy::lp64());
        assert!(matches!(
            engine.layout(foreign_id),
            Err(LayoutError::Type(_))
        ));
    }
}
