//! Compose canonical promoted field paths into checked constants or expressions.
//! Captured values are supplied in source order; only pure aggregate composition
//! occurs here. Physical paths and selected union fields come from source lookup.
use super::paths::{self, PathStep};
use jai_ir::{ConstantKind, ConstantValue, ValueExpr};
use jai_types::{FieldId, RecordKind, TypeError, TypeId, TypeKind, TypeView};
use std::collections::HashMap;

pub(crate) trait FieldDefaults {
    type Error;
    /// Complete default, including source initializer and default overrides.
    fn complete(&mut self, field: FieldId) -> Result<ConstantValue, Self::Error>;
    /// A source-provided aggregate initializer/override to overlay. None means
    /// construction uses the actual selected branch's individual field defaults.
    /// This must not turn an arbitrary default evaluation error into None.
    fn partial(&mut self, field: FieldId) -> Result<Option<ConstantValue>, Self::Error>;
    /// A genuine element type default in the declaring array field's environment.
    fn array_element(&mut self, field: FieldId, ty: TypeId) -> Result<ConstantValue, Self::Error>;
}

#[derive(Debug)]
pub(crate) enum BuildError<E> {
    Type(TypeError),
    Default(E),
    DefaultType { expected: TypeId, actual: TypeId },
    MalformedDefault,
    MissingUnionAlternative,
    MissingArrayDefault,
    Budget,
}

#[derive(Clone, Default)]
struct Node {
    value: Option<usize>,
    children: HashMap<PathStep, Node>,
}

/// A source initializer can enter the pure composition only as an already
/// evaluated typed constant or an exact immutable Bound leaf.
/// General "static" arithmetic is not enough: it may trap when evaluated.
#[derive(Clone)]
pub(super) struct PreparedLeaf(ValueExpr);
impl PreparedLeaf {
    pub fn constant(value: ConstantValue) -> Self {
        Self(value.into_expression())
    }
    pub fn bound(binding: jai_ir::ExpressionBindingId, ty: TypeId) -> Self {
        Self(ValueExpr::Bound { binding, ty })
    }
}

#[derive(Clone)]
pub(crate) struct PreparedLiteral {
    root: TypeId,
    tree: Node,
    leaf_types: Vec<TypeId>,
}
impl PreparedLiteral {
    /// Resolve and validate every path before lowering any source initializer.
    pub(crate) fn new(
        types: &dyn TypeView,
        root: TypeId,
        leaves: Vec<(Vec<FieldId>, TypeId)>,
    ) -> Result<Self, paths::PathError> {
        paths::check_budget(leaves.iter().map(|(path, _)| path.len()))?;
        Self::with_paths(
            types,
            root,
            leaves
                .into_iter()
                .map(|(path, ty)| (path.into_iter().map(PathStep::Field).collect(), ty))
                .collect(),
        )
    }
    pub(crate) fn with_paths(
        types: &dyn TypeView,
        root: TypeId,
        leaves: Vec<(Vec<PathStep>, TypeId)>,
    ) -> Result<Self, paths::PathError> {
        paths::check_budget(leaves.iter().map(|(path, _)| path.len()))?;
        paths::check_steps(
            types,
            root,
            &leaves
                .iter()
                .map(|(path, ty)| (path.as_slice(), *ty))
                .collect::<Vec<_>>(),
        )?;
        let mut tree = Node::default();
        let mut leaf_types = Vec::with_capacity(leaves.len());
        for (index, (path, ty)) in leaves.into_iter().enumerate() {
            let mut node = &mut tree;
            for step in path {
                node = node.children.entry(step).or_default();
            }
            node.value = Some(index);
            leaf_types.push(ty);
        }
        Ok(Self {
            root,
            tree,
            leaf_types,
        })
    }
    /// Leaf types are checked facts; this validates only ready source defaults.
    pub(crate) fn validate_defaults<D: FieldDefaults>(
        &self,
        types: &dyn TypeView,
        defaults: &mut D,
    ) -> Result<(), BuildError<D::Error>> {
        self.clone()
            .compose_inner(types, self.leaf_types.clone(), defaults)
            .map(|_: TypeId| ())
    }
    /// Producers of captured values execute once in source order in the
    /// enclosing Bind; only pure composition occurs here.
    pub(super) fn compose<D: FieldDefaults>(
        self,
        types: &dyn TypeView,
        values: Vec<PreparedLeaf>,
        defaults: &mut D,
    ) -> Result<ValueExpr, BuildError<D::Error>> {
        self.compose_inner(
            types,
            values.into_iter().map(|value| value.0).collect(),
            defaults,
        )
    }
    pub(crate) fn compose_constants<D: FieldDefaults>(
        self,
        types: &dyn TypeView,
        values: Vec<ConstantValue>,
        defaults: &mut D,
    ) -> Result<ConstantValue, BuildError<D::Error>> {
        self.compose_inner(types, values, defaults)
    }
    fn compose_inner<D: FieldDefaults, V: ConstructedValue>(
        self,
        types: &dyn TypeView,
        values: Vec<V>,
        defaults: &mut D,
    ) -> Result<V, BuildError<D::Error>> {
        if values.len() != self.leaf_types.len() {
            return Err(BuildError::MalformedDefault);
        }
        for (value, expected) in values.iter().zip(self.leaf_types) {
            let actual = value.type_id(types);
            if actual != expected {
                return Err(BuildError::DefaultType { expected, actual });
            }
        }
        Composer {
            types,
            values: values.into_iter().map(Some).collect(),
            defaults,
            remaining: 65_536,
        }
        .node(self.root, self.tree, None, None, 0)
    }
}
trait ConstructedValue: Sized {
    fn type_id(&self, types: &dyn TypeView) -> TypeId;
    fn constant(value: ConstantValue) -> Self;
    fn record(ty: TypeId, initializers: Vec<(FieldId, Self)>) -> Self;
    fn union(ty: TypeId, field: FieldId, value: Self) -> Self;
    fn array(ty: TypeId, elements: Vec<Self>) -> Self;
}
impl ConstructedValue for TypeId {
    fn type_id(&self, _: &dyn TypeView) -> TypeId {
        *self
    }
    fn constant(value: ConstantValue) -> Self {
        value.ty
    }
    fn record(ty: TypeId, _: Vec<(FieldId, Self)>) -> Self {
        ty
    }
    fn union(ty: TypeId, _: FieldId, _: Self) -> Self {
        ty
    }
    fn array(ty: TypeId, _: Vec<Self>) -> Self {
        ty
    }
}
impl ConstructedValue for ValueExpr {
    fn array(ty: TypeId, elements: Vec<Self>) -> Self {
        Self::Array { ty, elements }
    }
    fn type_id(&self, types: &dyn TypeView) -> TypeId {
        self.type_id(types)
    }
    fn constant(value: ConstantValue) -> Self {
        value.into_expression()
    }
    fn record(ty: TypeId, initializers: Vec<(FieldId, Self)>) -> Self {
        Self::RecordBuild { ty, initializers }
    }
    fn union(ty: TypeId, field: FieldId, value: Self) -> Self {
        Self::Union {
            ty,
            field,
            value: Box::new(value),
        }
    }
}
impl ConstructedValue for ConstantValue {
    fn array(ty: TypeId, elements: Vec<Self>) -> Self {
        Self {
            ty,
            kind: ConstantKind::Array(elements),
        }
    }
    fn type_id(&self, _types: &dyn TypeView) -> TypeId {
        self.ty
    }
    fn constant(value: ConstantValue) -> Self {
        value
    }
    fn record(ty: TypeId, initializers: Vec<(FieldId, Self)>) -> Self {
        Self {
            ty,
            kind: ConstantKind::Record(initializers.into_iter().map(|(_, value)| value).collect()),
        }
    }
    fn union(ty: TypeId, field: FieldId, value: Self) -> Self {
        Self {
            ty,
            kind: ConstantKind::Union {
                field,
                value: Box::new(value),
            },
        }
    }
}
struct ArrayShape {
    ty: TypeId,
    element: TypeId,
    count: u64,
}
struct Composer<'a, D, V> {
    types: &'a dyn TypeView,
    values: Vec<Option<V>>,
    defaults: &'a mut D,
    remaining: usize,
}
impl<D: FieldDefaults, V: ConstructedValue> Composer<'_, D, V> {
    fn node(
        &mut self,
        ty: TypeId,
        mut node: Node,
        inherited: Option<ConstantValue>,
        array_context: Option<FieldId>,
        depth: usize,
    ) -> Result<V, BuildError<D::Error>> {
        if depth >= 128 || self.remaining == 0 {
            return Err(BuildError::Budget);
        }
        self.remaining -= 1;
        if let Some(index) = node.value {
            return self
                .values
                .get_mut(index)
                .and_then(Option::take)
                .ok_or(BuildError::MalformedDefault);
        }
        if let TypeKind::FixedArray { element, count } =
            *self.types.kind(ty).map_err(BuildError::Type)?
        {
            return self.array_node(
                ArrayShape { ty, element, count },
                node,
                inherited,
                array_context,
                depth,
            );
        }
        let definition = self
            .types
            .record_storage_definition(ty)
            .map_err(BuildError::Type)?;
        if let Some(value) = &inherited
            && value.ty != ty
        {
            return Err(BuildError::DefaultType {
                expected: ty,
                actual: value.ty,
            });
        }
        if definition.kind == RecordKind::Union {
            let mut children = node.children.into_iter();
            let Some((field, child)) = children.next() else {
                return Err(BuildError::MissingUnionAlternative);
            };
            if children.next().is_some() {
                return Err(BuildError::MalformedDefault);
            }
            let PathStep::Field(field) = field else {
                return Err(BuildError::MalformedDefault);
            };
            let child_ty = self
                .types
                .validate_field(ty, field)
                .map_err(BuildError::Type)?;
            let inherited = self.project(inherited.as_ref(), field)?;
            let value = self.child(field, child_ty, child, inherited, depth + 1)?;
            return Ok(V::union(ty, field, value));
        }
        if definition.fields.len() > self.remaining {
            return Err(BuildError::Budget);
        }
        let mut initializers = Vec::with_capacity(definition.fields.len());
        for index in 0..definition.fields.len() {
            let field = self.types.field(ty, index).map_err(BuildError::Type)?;
            let inherited = self.project(inherited.as_ref(), field.id)?;
            let value = match node.children.remove(&PathStep::Field(field.id)) {
                Some(child) => self.child(field.id, field.ty, child, inherited, depth + 1)?,
                None => {
                    let value = match inherited {
                        Some(value) => value,
                        None => self
                            .defaults
                            .complete(field.id)
                            .map_err(BuildError::Default)?,
                    };
                    self.default_expression(value, depth + 1)?
                }
            };
            let actual = value.type_id(self.types);
            if actual != field.ty {
                return Err(BuildError::DefaultType {
                    expected: field.ty,
                    actual,
                });
            }
            initializers.push((field.id, value));
        }
        if !node.children.is_empty() {
            return Err(BuildError::MalformedDefault);
        }
        Ok(V::record(ty, initializers))
    }
    fn array_node(
        &mut self,
        shape: ArrayShape,
        mut node: Node,
        inherited: Option<ConstantValue>,
        context: Option<FieldId>,
        depth: usize,
    ) -> Result<V, BuildError<D::Error>> {
        let ArrayShape { ty, element, count } = shape;
        let count = usize::try_from(count).map_err(|_| BuildError::Budget)?;
        if count > self.remaining {
            return Err(BuildError::Budget);
        }
        if let Some(value) = &inherited {
            if value.ty != ty {
                return Err(BuildError::DefaultType {
                    expected: ty,
                    actual: value.ty,
                });
            }
            match &value.kind {
                ConstantKind::Array(values)
                    if values.len() == count && values.iter().all(|value| value.ty == element) => {}
                ConstantKind::Zero => {}
                _ => return Err(BuildError::MalformedDefault),
            }
        }
        if inherited.is_none() && context.is_none() {
            return Err(BuildError::MissingArrayDefault);
        }
        let mut elements = Vec::with_capacity(count);
        for index in 0..count {
            let child_default = match &inherited {
                Some(ConstantValue {
                    kind: ConstantKind::Array(values),
                    ..
                }) => Some(values[index].clone()),
                Some(ConstantValue {
                    kind: ConstantKind::Zero,
                    ..
                }) => Some(ConstantValue {
                    ty: element,
                    kind: ConstantKind::Zero,
                }),
                _ => None,
            };
            let value = match node.children.remove(&PathStep::Element {
                owner: ty,
                index: index as u64,
            }) {
                Some(child) => {
                    let default = if child.value.is_some() {
                        None
                    } else {
                        child_default
                    };
                    self.node(element, child, default, context, depth + 1)?
                }
                None => {
                    let value = match child_default {
                        Some(value) => value,
                        None => self
                            .defaults
                            .array_element(context.ok_or(BuildError::MissingArrayDefault)?, element)
                            .map_err(BuildError::Default)?,
                    };
                    self.default_expression(value, depth + 1)?
                }
            };
            let actual = value.type_id(self.types);
            if actual != element {
                return Err(BuildError::DefaultType {
                    expected: element,
                    actual,
                });
            }
            elements.push(value);
        }
        if !node.children.is_empty() {
            return Err(BuildError::MalformedDefault);
        }
        Ok(V::array(ty, elements))
    }
    fn child(
        &mut self,
        field: FieldId,
        ty: TypeId,
        child: Node,
        inherited: Option<ConstantValue>,
        depth: usize,
    ) -> Result<V, BuildError<D::Error>> {
        if child.value.is_none()
            && let TypeKind::FixedArray { count, .. } =
                *self.types.kind(ty).map_err(BuildError::Type)?
            && count > self.remaining.saturating_sub(1) as u64
        {
            return Err(BuildError::Budget);
        }
        let inherited = if child.value.is_some() {
            None
        } else {
            match inherited {
                Some(value) => Some(value),
                None => self.defaults.partial(field).map_err(BuildError::Default)?,
            }
        };
        self.node(ty, child, inherited, Some(field), depth)
    }
    fn default_expression(
        &mut self,
        value: ConstantValue,
        depth: usize,
    ) -> Result<V, BuildError<D::Error>> {
        let mut pending = vec![(&value, depth)];
        while let Some((value, depth)) = pending.pop() {
            if depth >= 128 || self.remaining == 0 {
                return Err(BuildError::Budget);
            }
            self.remaining -= 1;
            match &value.kind {
                ConstantKind::Record(values) | ConstantKind::Array(values) => {
                    if values.len() > self.remaining {
                        return Err(BuildError::Budget);
                    }
                    pending.extend(values.iter().map(|value| (value, depth + 1)));
                }
                ConstantKind::Union { value, .. } | ConstantKind::Distinct(value) => {
                    pending.push((value, depth + 1))
                }
                _ => {}
            }
        }
        Ok(V::constant(value))
    }
    fn project(
        &self,
        value: Option<&ConstantValue>,
        field: FieldId,
    ) -> Result<Option<ConstantValue>, BuildError<D::Error>> {
        let Some(value) = value else {
            return Ok(None);
        };
        let ty = self
            .types
            .validate_field(value.ty, field)
            .map_err(BuildError::Type)?;
        let definition = self
            .types
            .record_storage_definition(value.ty)
            .map_err(BuildError::Type)?;
        let result = match &value.kind {
            ConstantKind::Zero => Some(ConstantValue {
                ty,
                kind: ConstantKind::Zero,
            }),
            ConstantKind::Record(fields) if definition.kind == RecordKind::Struct => {
                if fields.len() != definition.fields.len()
                    || fields
                        .iter()
                        .zip(&definition.fields)
                        .any(|(value, &ty)| value.ty != ty)
                {
                    return Err(BuildError::MalformedDefault);
                }
                Some(fields[field.index()].clone())
            }
            ConstantKind::Union {
                field: selected,
                value: child,
            } if definition.kind == RecordKind::Union => {
                let selected_ty = self
                    .types
                    .validate_field(value.ty, *selected)
                    .map_err(BuildError::Type)?;
                if selected_ty != child.ty {
                    return Err(BuildError::MalformedDefault);
                }
                (*selected == field).then(|| (**child).clone())
            }
            _ => return Err(BuildError::MalformedDefault),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{Integer, IntegerType, ScalarType, TypeRegistry};

    #[derive(Default)]
    struct Defaults {
        complete: HashMap<FieldId, ConstantValue>,
        partial: HashMap<FieldId, ConstantValue>,
        calls: Vec<FieldId>,
        elements: HashMap<(FieldId, TypeId), ConstantValue>,
        failure: Option<FieldId>,
    }
    impl FieldDefaults for Defaults {
        type Error = &'static str;
        fn complete(&mut self, field: FieldId) -> Result<ConstantValue, Self::Error> {
            self.calls.push(field);
            if self.failure == Some(field) {
                return Err("source default failed");
            }
            self.complete
                .get(&field)
                .cloned()
                .ok_or("missing source default")
        }
        fn array_element(
            &mut self,
            field: FieldId,
            ty: TypeId,
        ) -> Result<ConstantValue, Self::Error> {
            self.elements
                .get(&(field, ty))
                .cloned()
                .ok_or("missing checked array element default")
        }
        fn partial(&mut self, field: FieldId) -> Result<Option<ConstantValue>, Self::Error> {
            if self.failure == Some(field) {
                return Err("source default failed");
            }
            Ok(self.partial.get(&field).cloned())
        }
    }
    fn int(types: &TypeRegistry, value: i128) -> ConstantValue {
        ConstantValue {
            ty: types.scalar(ScalarType::Int(IntegerType::S64)),
            kind: ConstantKind::Int(Integer::checked(IntegerType::S64, value).unwrap()),
        }
    }
    fn record(types: &mut TypeRegistry, kind: RecordKind, fields: Vec<TypeId>) -> TypeId {
        let ty = types.reserve_record(kind);
        types.define_record(ty, fields).unwrap();
        ty
    }
    fn field(types: &TypeRegistry, owner: TypeId, index: usize) -> FieldId {
        types.field(owner, index).unwrap().id
    }
    fn checked(types: &TypeRegistry, expression: &ValueExpr) {
        jai_ir::verify_expression(
            types,
            expression,
            &HashMap::new(),
            &[],
            &jai_ir::Places::default(),
        )
        .unwrap();
    }
    fn field_value(expression: &ValueExpr, field: FieldId) -> &ValueExpr {
        let ValueExpr::RecordBuild { initializers, .. } = expression else {
            panic!("physical struct build required");
        };
        &initializers.iter().find(|(id, _)| *id == field).unwrap().1
    }
    fn integer(expression: &ValueExpr) -> i128 {
        let ValueExpr::Int(value) = expression else {
            panic!("integer required");
        };
        let jai_ir::IntExprKind::Constant(value) = value.kind() else {
            panic!("constant integer required");
        };
        value.value()
    }

    #[test]
    fn partial_selected_union_branch_uses_real_sibling_default() {
        let mut types = TypeRegistry::new();
        let int_ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let branch = record(&mut types, RecordKind::Struct, vec![int_ty, int_ty]);
        let union = record(&mut types, RecordKind::Union, vec![branch, int_ty]);
        let root = record(&mut types, RecordKind::Struct, vec![int_ty, union]);
        let tag = field(&types, root, 0);
        let embedded = field(&types, root, 1);
        let selected = field(&types, union, 0);
        let alternative = field(&types, union, 1);
        let x = field(&types, branch, 0);
        let y = field(&types, branch, 1);
        let plan = PreparedLiteral::new(
            &types,
            root,
            vec![(vec![tag], int_ty), (vec![embedded, selected, x], int_ty)],
        )
        .unwrap();
        let mut defaults = Defaults::default();
        defaults.complete.insert(y, int(&types, 22));
        defaults.failure = Some(alternative);
        let expression = plan
            .compose(
                &types,
                vec![
                    PreparedLeaf::constant(int(&types, 3)),
                    PreparedLeaf::constant(int(&types, 20)),
                ],
                &mut defaults,
            )
            .unwrap();
        checked(&types, &expression);
        let ValueExpr::Union {
            field: actual,
            value,
            ..
        } = field_value(&expression, embedded)
        else {
            panic!("selected union required");
        };
        assert_eq!(*actual, selected);
        assert_eq!(
            integer(field_value(value, x)) + integer(field_value(value, y)),
            42
        );
        assert_eq!(defaults.calls, [y]);
    }

    #[test]
    fn source_leaf_indices_survive_physical_field_grouping() {
        let mut types = TypeRegistry::new();
        let int_ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let child = record(&mut types, RecordKind::Struct, vec![int_ty, int_ty]);
        let root = record(&mut types, RecordKind::Struct, vec![child, int_ty]);
        let child_field = field(&types, root, 0);
        let middle = field(&types, root, 1);
        let x = field(&types, child, 0);
        let y = field(&types, child, 1);
        let plan = PreparedLiteral::new(
            &types,
            root,
            vec![
                (vec![child_field, x], int_ty),
                (vec![middle], int_ty),
                (vec![child_field, y], int_ty),
            ],
        )
        .unwrap();
        let values = [1, 2, 3]
            .map(|n| PreparedLeaf::constant(int(&types, n)))
            .to_vec();
        let expression = plan
            .compose(&types, values, &mut Defaults::default())
            .unwrap();
        checked(&types, &expression);
        let child_value = field_value(&expression, child_field);
        assert_eq!(integer(field_value(child_value, x)), 1);
        assert_eq!(integer(field_value(&expression, middle)), 2);
        assert_eq!(integer(field_value(child_value, y)), 3);
    }

    #[test]
    fn partial_named_aggregate_preserves_checked_parent_initializer() {
        let mut types = TypeRegistry::new();
        let int_ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let child = record(&mut types, RecordKind::Struct, vec![int_ty, int_ty]);
        let root = record(&mut types, RecordKind::Struct, vec![child]);
        let parent = field(&types, root, 0);
        let x = field(&types, child, 0);
        let y = field(&types, child, 1);
        let mut defaults = Defaults::default();
        defaults.partial.insert(
            parent,
            ConstantValue {
                ty: child,
                kind: ConstantKind::Record(vec![int(&types, 1), int(&types, 7)]),
            },
        );
        let expression = PreparedLiteral::new(&types, root, vec![(vec![parent, x], int_ty)])
            .unwrap()
            .compose(
                &types,
                vec![PreparedLeaf::constant(int(&types, 35))],
                &mut defaults,
            )
            .unwrap();
        checked(&types, &expression);
        let child_value = field_value(&expression, parent);
        assert_eq!(
            integer(field_value(child_value, x)) + integer(field_value(child_value, y)),
            42
        );
        assert!(defaults.calls.is_empty());
    }

    #[test]
    fn same_union_type_in_distinct_physical_fields_can_choose_different_branches() {
        let mut types = TypeRegistry::new();
        let int_ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let union = record(&mut types, RecordKind::Union, vec![int_ty, int_ty]);
        let root = record(&mut types, RecordKind::Struct, vec![union, union]);
        let first = field(&types, root, 0);
        let second = field(&types, root, 1);
        let a = field(&types, union, 0);
        let b = field(&types, union, 1);
        let expression = PreparedLiteral::new(
            &types,
            root,
            vec![(vec![first, a], int_ty), (vec![second, b], int_ty)],
        )
        .unwrap()
        .compose(
            &types,
            vec![
                PreparedLeaf::constant(int(&types, 20)),
                PreparedLeaf::constant(int(&types, 22)),
            ],
            &mut Defaults::default(),
        )
        .unwrap();
        checked(&types, &expression);
        assert!(matches!(field_value(&expression,first),ValueExpr::Union{field,..} if *field==a));
        assert!(matches!(field_value(&expression,second),ValueExpr::Union{field,..} if *field==b));
    }

    #[test]
    fn unmentioned_zero_union_storage_has_no_fabricated_active_alternative() {
        let mut types = TypeRegistry::new();
        let int_ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let union = record(&mut types, RecordKind::Union, vec![int_ty, int_ty]);
        let root = record(&mut types, RecordKind::Struct, vec![union]);
        let embedded = field(&types, root, 0);
        let mut defaults = Defaults::default();
        defaults.complete.insert(
            embedded,
            ConstantValue {
                ty: union,
                kind: ConstantKind::Zero,
            },
        );
        let expression = PreparedLiteral::new(&types, root, vec![])
            .unwrap()
            .compose(&types, vec![], &mut defaults)
            .unwrap();
        checked(&types, &expression);
        assert!(matches!(field_value(&expression,embedded),ValueExpr::Zero(ty) if *ty==union));
    }

    #[test]
    fn unrelated_default_error_is_propagated() {
        let mut types = TypeRegistry::new();
        let int_ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let child = record(&mut types, RecordKind::Struct, vec![int_ty, int_ty]);
        let root = record(&mut types, RecordKind::Struct, vec![child]);
        let embedded = field(&types, root, 0);
        let x = field(&types, child, 0);
        let mut defaults = Defaults {
            failure: Some(embedded),
            ..Default::default()
        };
        let result = PreparedLiteral::new(&types, root, vec![(vec![embedded, x], int_ty)])
            .unwrap()
            .compose(
                &types,
                vec![PreparedLeaf::constant(int(&types, 20))],
                &mut defaults,
            );
        assert!(matches!(
            result,
            Err(BuildError::Default("source default failed"))
        ));
    }

    #[test]
    fn selected_branch_cannot_project_a_foreign_union_field_default() {
        let mut types = TypeRegistry::new();
        let int_ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let child = record(&mut types, RecordKind::Struct, vec![int_ty]);
        let union = record(&mut types, RecordKind::Union, vec![child]);
        let foreign = record(&mut types, RecordKind::Union, vec![child]);
        let root = record(&mut types, RecordKind::Struct, vec![union]);
        let embedded = field(&types, root, 0);
        let selected = field(&types, union, 0);
        let x = field(&types, child, 0);
        let mut defaults = Defaults::default();
        defaults.partial.insert(
            embedded,
            ConstantValue {
                ty: union,
                kind: ConstantKind::Union {
                    field: field(&types, foreign, 0),
                    value: Box::new(ConstantValue {
                        ty: child,
                        kind: ConstantKind::Record(vec![int(&types, 42)]),
                    }),
                },
            },
        );
        let result =
            PreparedLiteral::new(&types, root, vec![(vec![embedded, selected, x], int_ty)])
                .unwrap()
                .compose(
                    &types,
                    vec![PreparedLeaf::constant(int(&types, 20))],
                    &mut defaults,
                );
        assert!(matches!(result, Err(BuildError::Type(_))));
    }
    #[test]
    fn constant_composition_preserves_selected_field_and_checked_defaults() {
        let mut types = TypeRegistry::new();
        let int_ty = types.scalar(ScalarType::Int(IntegerType::S64));
        let branch = record(&mut types, RecordKind::Struct, vec![int_ty, int_ty]);
        let union = record(&mut types, RecordKind::Union, vec![branch, int_ty]);
        let root = record(&mut types, RecordKind::Struct, vec![union]);
        let embedded = field(&types, root, 0);
        let selected = field(&types, union, 0);
        let x = field(&types, branch, 0);
        let y = field(&types, branch, 1);
        let mut defaults = Defaults::default();
        defaults.complete.insert(y, int(&types, 22));
        let value = PreparedLiteral::new(&types, root, vec![(vec![embedded, selected, x], int_ty)])
            .unwrap()
            .compose_constants(&types, vec![int(&types, 20)], &mut defaults)
            .unwrap();
        jai_ir::Global::new_typed(0, value.clone(), &types).unwrap();
        let ConstantKind::Record(fields) = value.kind else {
            panic!("record constant required");
        };
        let ConstantKind::Union { field, value } = &fields[0].kind else {
            panic!("actual selected union field required");
        };
        assert_eq!(*field, selected);
        let ConstantKind::Record(values) = &value.kind else {
            panic!("selected struct constant required");
        };
        assert_eq!(values, &vec![int(&types, 20), int(&types, 22)]);
    }
    fn array_path(field: FieldId, array: TypeId, index: u64) -> Vec<PathStep> {
        vec![
            PathStep::Field(field),
            PathStep::Element {
                owner: array,
                index,
            },
        ]
    }
    fn constant_array(value: &ConstantValue) -> &[ConstantValue] {
        let ConstantKind::Record(fields) = &value.kind else {
            panic!("record required");
        };
        let ConstantKind::Array(values) = &fields[0].kind else {
            panic!("checked array required");
        };
        values
    }

    #[test]
    fn array_overlay_preserves_checked_untouched_values() {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let array = types.fixed_array(integer, 3).unwrap();
        let root = record(&mut types, RecordKind::Struct, vec![array]);
        let items = field(&types, root, 0);
        let plan =
            PreparedLiteral::with_paths(&types, root, vec![(array_path(items, array, 1), integer)])
                .unwrap();
        let mut defaults = Defaults::default();
        defaults.partial.insert(
            items,
            ConstantValue {
                ty: array,
                kind: ConstantKind::Array(vec![int(&types, 10), int(&types, 11), int(&types, 12)]),
            },
        );
        // The supplied aggregate is projected without an element type default.
        plan.validate_defaults(&types, &mut defaults).unwrap();
        let value = plan
            .compose_constants(&types, vec![int(&types, 42)], &mut defaults)
            .unwrap();
        assert_eq!(
            constant_array(&value),
            &[int(&types, 10), int(&types, 42), int(&types, 12)]
        );
        jai_ir::Global::new_typed(0, value, &types).unwrap();
    }

    #[test]
    fn array_omission_uses_genuine_element_default_and_propagates_failure() {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let array = types.fixed_array(integer, 2).unwrap();
        let root = record(&mut types, RecordKind::Struct, vec![array]);
        let items = field(&types, root, 0);
        let plan =
            PreparedLiteral::with_paths(&types, root, vec![(array_path(items, array, 1), integer)])
                .unwrap();
        let mut defaults = Defaults::default();
        assert!(matches!(
            plan.validate_defaults(&types, &mut defaults),
            Err(BuildError::Default("missing checked array element default"))
        ));
        defaults.elements.insert((items, integer), int(&types, 7));
        let value = plan
            .clone()
            .compose_constants(&types, vec![int(&types, 42)], &mut defaults)
            .unwrap();
        assert_eq!(constant_array(&value), &[int(&types, 7), int(&types, 42)]);
        defaults.failure = Some(items);
        assert!(matches!(
            plan.validate_defaults(&types, &mut defaults),
            Err(BuildError::Default("source default failed"))
        ));
    }

    #[test]
    fn arrays_reject_malformed_overlays_before_composition() {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let array = types.fixed_array(integer, 2).unwrap();
        let root = record(&mut types, RecordKind::Struct, vec![array]);
        let items = field(&types, root, 0);
        let plan =
            PreparedLiteral::with_paths(&types, root, vec![(array_path(items, array, 1), integer)])
                .unwrap();
        let mut defaults = Defaults::default();
        defaults.partial.insert(
            items,
            ConstantValue {
                ty: array,
                kind: ConstantKind::Array(vec![int(&types, 0)]),
            },
        );
        assert!(matches!(
            plan.validate_defaults(&types, &mut defaults),
            Err(BuildError::MalformedDefault)
        ));
        let boolean = types.scalar(ScalarType::Bool);
        defaults.partial.insert(
            items,
            ConstantValue {
                ty: array,
                kind: ConstantKind::Array(vec![
                    ConstantValue {
                        ty: boolean,
                        kind: ConstantKind::Bool(false),
                    },
                    int(&types, 0),
                ]),
            },
        );
        assert!(matches!(
            plan.validate_defaults(&types, &mut defaults),
            Err(BuildError::MalformedDefault)
        ));
    }

    #[test]
    fn independent_array_union_elements_keep_selected_alternatives() {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let branch = record(&mut types, RecordKind::Struct, vec![integer, integer]);
        let union = record(&mut types, RecordKind::Union, vec![branch, integer]);
        let array = types.fixed_array(union, 2).unwrap();
        let root = record(&mut types, RecordKind::Struct, vec![array]);
        let items = field(&types, root, 0);
        let structured = field(&types, union, 0);
        let scalar = field(&types, union, 1);
        let x = field(&types, branch, 0);
        let y = field(&types, branch, 1);
        let mut first = array_path(items, array, 0);
        first.extend([PathStep::Field(structured), PathStep::Field(x)]);
        let mut second = array_path(items, array, 1);
        second.push(PathStep::Field(scalar));
        let plan =
            PreparedLiteral::with_paths(&types, root, vec![(first, integer), (second, integer)])
                .unwrap();
        let mut defaults = Defaults::default();
        defaults.complete.insert(y, int(&types, 22));
        defaults.failure = Some(scalar);
        let value = plan
            .compose_constants(
                &types,
                vec![int(&types, 20), int(&types, 42)],
                &mut defaults,
            )
            .unwrap();
        let values = constant_array(&value);
        assert!(matches!(&values[0].kind, ConstantKind::Union {field, ..} if *field==structured));
        assert!(matches!(&values[1].kind, ConstantKind::Union {field, ..} if *field==scalar));
        assert_eq!(defaults.calls, vec![y]);
        jai_ir::Global::new_typed(0, value, &types).unwrap();
    }

    #[test]
    fn oversized_array_rejects_before_default_provider_or_vector_allocation() {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let array = types.fixed_array(integer, 65_537).unwrap();
        let root = record(&mut types, RecordKind::Struct, vec![array]);
        let items = field(&types, root, 0);
        let plan =
            PreparedLiteral::with_paths(&types, root, vec![(array_path(items, array, 0), integer)])
                .unwrap();
        let mut defaults = Defaults {
            failure: Some(items),
            ..Defaults::default()
        };
        assert!(matches!(
            plan.validate_defaults(&types, &mut defaults),
            Err(BuildError::Budget)
        ));
    }
}
