//! Canonical bound expression identity excludes diagnostic source positions.
use super::*;
mod encoding;
mod retained_metadata;
pub use encoding::{WeakFloatAdmissionError, WeakFloatEncodingError};

/// Exact contextual constant identity. Nodes contain validated literals and
/// numeric operations, never rounded fallback values or diagnostic locations.
#[derive(Clone)]
pub struct WeakFloatKey(std::sync::Arc<KeyNode>);

struct KeyNode {
    tag: Node,
    children: Box<[WeakFloatKey]>,
    fingerprint: u64,
}

impl WeakFloatKey {
    fn new(tag: Node, children: Vec<Self>) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        tag.hash(&mut hasher);
        children.len().hash(&mut hasher);
        for child in &children {
            child.0.fingerprint.hash(&mut hasher);
        }
        Self(std::sync::Arc::new(KeyNode {
            tag,
            children: children.into_boxed_slice(),
            fingerprint: hasher.finish(),
        }))
    }

    #[cfg(test)]
    pub(super) fn node_count(&self) -> usize {
        let mut visited = std::collections::HashSet::new();
        let mut pending = vec![self];
        while let Some(key) = pending.pop() {
            if visited.insert(std::sync::Arc::as_ptr(&key.0)) {
                pending.extend(key.0.children.iter());
            }
        }
        visited.len()
    }
}

impl std::fmt::Debug for WeakFloatKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Expanding shared children here would recreate the exponential tree.
        f.debug_struct("WeakFloatKey")
            .field("fingerprint", &self.0.fingerprint)
            .finish_non_exhaustive()
    }
}

impl PartialEq for WeakFloatKey {
    fn eq(&self, other: &Self) -> bool {
        if std::sync::Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
        if self.0.fingerprint != other.0.fingerprint {
            return false;
        }
        let mut pending = vec![(self, other)];
        let mut visited = std::collections::HashSet::new();
        while let Some((left, right)) = pending.pop() {
            if std::sync::Arc::ptr_eq(&left.0, &right.0) {
                continue;
            }
            if left.0.fingerprint != right.0.fingerprint
                || left.0.tag != right.0.tag
                || left.0.children.len() != right.0.children.len()
            {
                return false;
            }
            if visited.insert((
                std::sync::Arc::as_ptr(&left.0),
                std::sync::Arc::as_ptr(&right.0),
            )) {
                pending.extend(left.0.children.iter().zip(right.0.children.iter()));
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_hash_collisions_still_compare_the_exact_child_structure() {
        let left_child = WeakFloatKey::new(Node::Bool(true), vec![]);
        let mut right_child = WeakFloatKey::new(Node::Bool(false), vec![]);
        std::sync::Arc::get_mut(&mut right_child.0)
            .unwrap()
            .fingerprint = left_child.0.fingerprint;
        let left = WeakFloatKey::new(Node::BoolNot, vec![left_child]);
        let right = WeakFloatKey::new(Node::BoolNot, vec![right_child]);
        assert_eq!(left.0.fingerprint, right.0.fingerprint);
        assert_ne!(left, right);
    }
}
impl Eq for WeakFloatKey {
}
impl std::hash::Hash for WeakFloatKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Equal semantic trees have equal cached hashes, regardless of sharing.
        // Hash collisions never substitute for the structural comparison above.
        std::hash::Hash::hash(&self.0.fingerprint, state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Node {
    FloatDefault(FloatType),
    Float(Option<FloatType>),
    Decimal(DecimalLiteral),
    FloatConstant(FloatValue),
    FloatNumber,
    FloatCast(FloatType),
    FloatNegate,
    FloatBinary(FloatOp),
    FloatConditional,
    Number(NumberType),
    Literal(i128),
    Integer(Integer),
    NumberBool,
    NumberFloat(CastMode),
    NumberCast(CastMode),
    NumberNegate(CheckMode),
    NumberComplement,
    NumberBinary(IntOp, CheckMode),
    NumberConditional,
    Bool(bool),
    BoolNumber,
    BoolFloat,
    BoolFloatCompare(Relation),
    BoolNot,
    BoolNumberCompare(Relation),
    BoolCompare(Equality),
    BoolAnd,
    BoolOr,
    BoolConditional,
}

pub(super) fn build(expression: &FloatExpr) -> WeakFloatKey {
    float(expression)
}
pub(super) fn with_default(key: WeakFloatKey, default: FloatType) -> WeakFloatKey {
    WeakFloatKey::new(Node::FloatDefault(default), vec![key])
}

fn float(expression: &FloatExpr) -> WeakFloatKey {
    use WeakFloatKey as Key;
    let kind = match &expression.kind {
        FloatKind::Bound(value) => return value.request_key().clone(),
        FloatKind::Decimal(value) => Key::new(Node::Decimal(value.clone()), vec![]),
        FloatKind::Constant(value) => Key::new(Node::FloatConstant(*value), vec![]),
        FloatKind::Number(value) => Key::new(Node::FloatNumber, vec![number(value)]),
        FloatKind::Cast(ty, value) => Key::new(Node::FloatCast(*ty), vec![float(value)]),
        FloatKind::Negate(value) => Key::new(Node::FloatNegate, vec![float(value)]),
        FloatKind::Binary(op, a, b) => Key::new(Node::FloatBinary(*op), vec![float(a), float(b)]),
        FloatKind::Conditional(value) => Key::new(
            Node::FloatConditional,
            vec![
                boolean(&value.condition),
                float(&value.then_value),
                float(&value.else_value),
            ],
        ),
    };
    Key::new(Node::Float(expression.ty), vec![kind])
}

fn number(expression: &NumberExpr) -> WeakFloatKey {
    use WeakFloatKey as Key;
    let kind = match &expression.kind {
        NumberKind::Literal(value) => Key::new(Node::Literal(*value), vec![]),
        NumberKind::Typed(value) => Key::new(Node::Integer(*value), vec![]),
        NumberKind::FromBool(value) => Key::new(Node::NumberBool, vec![boolean(value)]),
        NumberKind::FromFloat(mode, value, _) => {
            Key::new(Node::NumberFloat(*mode), vec![float(value)])
        }
        NumberKind::Cast(mode, value, _) => Key::new(Node::NumberCast(*mode), vec![number(value)]),
        NumberKind::Negate(value, _) => Key::new(
            Node::NumberNegate(expression.overflow_check),
            vec![number(value)],
        ),
        NumberKind::Complement(value) => Key::new(Node::NumberComplement, vec![number(value)]),
        NumberKind::Binary(op, a, b, _) => Key::new(
            Node::NumberBinary(*op, expression.overflow_check),
            vec![number(a), number(b)],
        ),
        NumberKind::Conditional(value) => Key::new(
            Node::NumberConditional,
            vec![
                boolean(&value.condition),
                number(&value.then_value),
                number(&value.else_value),
            ],
        ),
    };
    Key::new(Node::Number(expression.ty), vec![kind])
}

fn boolean(expression: &BoolExpr) -> WeakFloatKey {
    use WeakFloatKey as Key;
    match expression {
        BoolExpr::Constant(value) => Key::new(Node::Bool(*value), vec![]),
        BoolExpr::FromNumber(value) => Key::new(Node::BoolNumber, vec![number(value)]),
        BoolExpr::FromFloat(value) => Key::new(Node::BoolFloat, vec![float(value)]),
        BoolExpr::CompareFloats(op, a, b, _) => {
            Key::new(Node::BoolFloatCompare(*op), vec![float(a), float(b)])
        }
        BoolExpr::Not(value) => Key::new(Node::BoolNot, vec![boolean(value)]),
        BoolExpr::CompareNumbers(op, a, b) => {
            Key::new(Node::BoolNumberCompare(*op), vec![number(a), number(b)])
        }
        BoolExpr::CompareBools(op, a, b) => {
            Key::new(Node::BoolCompare(*op), vec![boolean(a), boolean(b)])
        }
        BoolExpr::And(a, b) => Key::new(Node::BoolAnd, vec![boolean(a), boolean(b)]),
        BoolExpr::Or(a, b) => Key::new(Node::BoolOr, vec![boolean(a), boolean(b)]),
        BoolExpr::Conditional(value) => Key::new(
            Node::BoolConditional,
            vec![
                boolean(&value.condition),
                boolean(&value.then_value),
                boolean(&value.else_value),
            ],
        ),
    }
}
