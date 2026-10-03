//! Versioned, exact semantic DAG encoding for immutable replay facts.
use super::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeakFloatEncodingError {
    NodeLimit,
    ByteLimit,
}
impl std::fmt::Display for WeakFloatEncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NodeLimit => "weak float encoding exceeds its node limit",
            Self::ByteLimit => "weak float encoding exceeds its byte limit",
        })
    }
}
impl std::error::Error for WeakFloatEncodingError {}

struct Output {
    bytes: Vec<u8>,
    limit: usize,
}
impl Output {
    fn append(&mut self, bytes: &[u8]) -> Result<(), WeakFloatEncodingError> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(WeakFloatEncodingError::ByteLimit);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn byte(&mut self, value: u8) -> Result<(), WeakFloatEncodingError> {
        self.append(&[value])
    }
    fn integer_type(&mut self, ty: IntegerType) -> Result<(), WeakFloatEncodingError> {
        self.append(&[ty.bits() as u8, u8::from(ty.signed())])
    }
    fn text(&mut self, text: &str) -> Result<(), WeakFloatEncodingError> {
        let length = u32::try_from(text.len()).map_err(|_| WeakFloatEncodingError::ByteLimit)?;
        self.append(&length.to_le_bytes())?;
        self.append(text.as_bytes())
    }
    fn tag(&mut self, node: &Node) -> Result<(), WeakFloatEncodingError> {
        match node {
            Node::FloatDefault(ty) => self.append(&[28, float_type(*ty)]),
            Node::Float(ty) => self.append(&[0, ty.map_or(0, float_type)]),
            Node::Decimal(value) => {
                self.byte(1)?;
                self.text(value.spelling())
            }
            Node::FloatConstant(value) => {
                self.append(&[2, float_type(value.ty())])?;
                self.append(&value.bits().to_le_bytes())
            }
            Node::FloatNumber => self.byte(3),
            Node::FloatCast(ty) => self.append(&[4, float_type(*ty)]),
            Node::FloatNegate => self.byte(5),
            Node::FloatBinary(op) => self.append(&[6, float_op(*op)]),
            Node::FloatConditional => self.byte(7),
            Node::Number(ty) => {
                self.byte(8)?;
                match ty {
                    NumberType::Literal => self.byte(0),
                    NumberType::Typed(ty) => {
                        self.byte(1)?;
                        self.integer_type(*ty)
                    }
                }
            }
            Node::Literal(value) => {
                self.byte(9)?;
                self.append(&value.to_le_bytes())
            }
            Node::Integer(value) => {
                self.byte(10)?;
                self.integer_type(value.ty())?;
                self.append(&value.bits().to_le_bytes())
            }
            Node::NumberBool => self.byte(11),
            Node::NumberFloat(mode) => self.append(&[12, cast_mode(*mode)]),
            Node::NumberCast(mode) => self.append(&[13, cast_mode(*mode)]),
            Node::NumberNegate(check) => self.append(&[14, check_mode(*check)]),
            Node::NumberComplement => self.byte(15),
            Node::NumberBinary(op, check) => self.append(&[16, int_op(*op), check_mode(*check)]),
            Node::NumberConditional => self.byte(17),
            Node::Bool(value) => self.append(&[18, u8::from(*value)]),
            Node::BoolNumber => self.byte(19),
            Node::BoolFloat => self.byte(20),
            Node::BoolFloatCompare(op) => self.append(&[21, relation(*op)]),
            Node::BoolNot => self.byte(22),
            Node::BoolNumberCompare(op) => self.append(&[23, relation(*op)]),
            Node::BoolCompare(op) => self.append(&[
                24,
                match op {
                    Equality::Equal => 0,
                    Equality::NotEqual => 1,
                },
            ]),
            Node::BoolAnd => self.byte(25),
            Node::BoolOr => self.byte(26),
            Node::BoolConditional => self.byte(27),
        }
    }
}

impl WeakFloatKey {
    /// Export exact semantic identity, with checked traversal and output limits.
    /// Equal keys encode identically regardless of source positions or sharing.
    pub fn canonical_bytes(
        &self,
        max_nodes: usize,
        max_bytes: usize,
    ) -> Result<Vec<u8>, WeakFloatEncodingError> {
        let mut output = Output {
            bytes: Vec::new(),
            limit: max_bytes,
        };
        // WFK + version, canonical record count, records, and final root ID.
        output.append(b"WFK\x01\0\0\0\0")?;
        let mut resolved = HashMap::new();
        let mut canonical = HashMap::<(Node, Vec<u32>), u32>::new();
        let mut pending = vec![(self, false)];
        let mut visited = 0usize;
        while let Some((key, finish)) = pending.pop() {
            let pointer = std::sync::Arc::as_ptr(&key.0);
            if resolved.contains_key(&pointer) {
                continue;
            }
            if !finish {
                if visited >= max_nodes {
                    return Err(WeakFloatEncodingError::NodeLimit);
                }
                visited += 1;
                pending.push((key, true));
                pending.extend(key.0.children.iter().rev().map(|child| (child, false)));
                continue;
            }
            let children: Vec<u32> = key
                .0
                .children
                .iter()
                .map(|child| resolved[&std::sync::Arc::as_ptr(&child.0)])
                .collect();
            // Reject a huge spelling before cloning it into the interning map.
            if let Node::Decimal(value) = &key.0.tag
                && value.spelling().len() > max_bytes
            {
                return Err(WeakFloatEncodingError::ByteLimit);
            }
            let signature = (key.0.tag.clone(), children);
            let id = if let Some(id) = canonical.get(&signature) {
                *id
            } else {
                let id = u32::try_from(canonical.len())
                    .map_err(|_| WeakFloatEncodingError::NodeLimit)?;
                output.tag(&signature.0)?;
                // The private grammar has at most three children per node.
                output.byte(signature.1.len() as u8)?;
                for child in &signature.1 {
                    output.append(&child.to_le_bytes())?;
                }
                canonical.insert(signature, id);
                id
            };
            resolved.insert(pointer, id);
        }
        let count =
            u32::try_from(canonical.len()).map_err(|_| WeakFloatEncodingError::NodeLimit)?;
        output.bytes[4..8].copy_from_slice(&count.to_le_bytes());
        output.append(&resolved[&std::sync::Arc::as_ptr(&self.0)].to_le_bytes())?;
        Ok(output.bytes)
    }
}

fn float_type(ty: FloatType) -> u8 {
    match ty {
        FloatType::F32 => 1,
        FloatType::F64 => 2,
    }
}
fn check_mode(mode: CheckMode) -> u8 {
    match mode {
        CheckMode::Enabled => 0,
        CheckMode::Disabled => 1,
    }
}
fn cast_mode(mode: CastMode) -> u8 {
    match mode {
        CastMode::Checked => 0,
        CastMode::Unchecked => 1,
        CastMode::Truncate => 2,
        CastMode::Force(jai_types::StorageBitcastStrength::EqualSize) => 3,
        CastMode::Force(jai_types::StorageBitcastStrength::Prefix) => 4,
    }
}
fn float_op(op: FloatOp) -> u8 {
    match op {
        FloatOp::Add => 0,
        FloatOp::Subtract => 1,
        FloatOp::Multiply => 2,
        FloatOp::Divide => 3,
        FloatOp::Remainder => 4,
    }
}
fn int_op(op: IntOp) -> u8 {
    match op {
        IntOp::Add => 0,
        IntOp::Subtract => 1,
        IntOp::Multiply => 2,
        IntOp::Divide => 3,
        IntOp::Remainder => 4,
        IntOp::BitAnd => 5,
        IntOp::BitOr => 6,
        IntOp::BitXor => 7,
        IntOp::ShiftLeft => 8,
        IntOp::ShiftRight => 9,
    }
}
fn relation(op: Relation) -> u8 {
    match op {
        Relation::Equal => 0,
        Relation::NotEqual => 1,
        Relation::Less => 2,
        Relation::LessEqual => 3,
        Relation::Greater => 4,
        Relation::GreaterEqual => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(text: &str) -> WeakFloatKey {
        let mut sources = jai_source::SourceMap::default();
        let source = sources.insert("encoding.jai".into(), format!("VALUE :: {text};"));
        let file = jai_syntax::parse_file(
            sources.get(source).unwrap(),
            &mut jai_source::Symbols::default(),
        )
        .unwrap();
        let jai_syntax::FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!()
        };
        let jai_syntax::FileDeclarationKind::Constant(constant) = &declaration.kind else {
            panic!()
        };
        let Value::WeakFloat(value) = evaluate_paths(&constant.initializer, |_, span| {
            Err(Diagnostic::new(span, "unknown"))
        })
        .unwrap() else {
            panic!()
        };
        value.request_key().clone()
    }

    #[test]
    fn version_one_simple_decimal_has_pinned_exact_bytes() {
        let expected = [
            b'W', b'F', b'K', 1, 2, 0, 0, 0, 1, 3, 0, 0, 0, b'1', b'.', b'0', 0, 0, 0, 1, 0, 0, 0,
            0, 1, 0, 0, 0,
        ];
        let value = key("1.0");
        assert_eq!(value.canonical_bytes(2, expected.len()).unwrap(), expected);
        assert_eq!(
            value.canonical_bytes(1, 100),
            Err(WeakFloatEncodingError::NodeLimit)
        );
        assert_eq!(
            value.canonical_bytes(2, expected.len() - 1),
            Err(WeakFloatEncodingError::ByteLimit)
        );
    }

    #[test]
    fn canonical_records_ignore_sharing_and_fingerprints_but_keep_exact_values() {
        let leaf = WeakFloatKey::new(Node::Bool(true), vec![]);
        let shared = WeakFloatKey::new(Node::BoolAnd, vec![leaf.clone(), leaf]);
        let expanded = WeakFloatKey::new(
            Node::BoolAnd,
            vec![
                WeakFloatKey::new(Node::Bool(true), vec![]),
                WeakFloatKey::new(Node::Bool(true), vec![]),
            ],
        );
        assert_eq!(
            shared.canonical_bytes(10, 100).unwrap(),
            expanded.canonical_bytes(10, 100).unwrap()
        );
        let mut collision = WeakFloatKey::new(Node::Bool(false), vec![]);
        std::sync::Arc::get_mut(&mut collision.0)
            .unwrap()
            .fingerprint = shared.0.fingerprint;
        assert_ne!(
            shared.canonical_bytes(10, 100).unwrap(),
            collision.canonical_bytes(10, 100).unwrap()
        );
        assert_eq!(
            key("1.0000000596046448").canonical_bytes(10, 100).unwrap(),
            key("   1.0000000596046448")
                .canonical_bytes(10, 100)
                .unwrap()
        );
        assert_ne!(
            key("1.0000000596046448").canonical_bytes(10, 100).unwrap(),
            key("1.0000000596046449").canonical_bytes(10, 100).unwrap()
        );
        for node in [
            Node::FloatConstant(FloatValue::F32(0x7fc00001)),
            Node::NumberFloat(CastMode::Checked),
            Node::NumberNegate(CheckMode::Enabled),
        ] {
            let changed = match node {
                Node::FloatConstant(_) => Node::FloatConstant(FloatValue::F32(0x7fc00002)),
                Node::NumberFloat(_) => Node::NumberFloat(CastMode::Truncate),
                Node::NumberNegate(_) => Node::NumberNegate(CheckMode::Disabled),
                _ => unreachable!(),
            };
            assert_ne!(
                WeakFloatKey::new(node, vec![])
                    .canonical_bytes(10, 100)
                    .unwrap(),
                WeakFloatKey::new(changed, vec![])
                    .canonical_bytes(10, 100)
                    .unwrap()
            );
        }
        let force_equal = WeakFloatKey::new(
            Node::NumberCast(CastMode::Force(
                jai_types::StorageBitcastStrength::EqualSize,
            )),
            vec![],
        );
        let force_prefix = WeakFloatKey::new(
            Node::NumberCast(CastMode::Force(jai_types::StorageBitcastStrength::Prefix)),
            vec![],
        );
        assert_ne!(
            force_equal.canonical_bytes(10, 100).unwrap(),
            force_prefix.canonical_bytes(10, 100).unwrap()
        );
    }

    #[test]
    fn deep_repeated_dag_encodes_with_bounded_growth_and_iterative_traversal() {
        let mut value = WeakFloatKey::new(Node::Bool(true), vec![]);
        for _ in 0..1000 {
            value = WeakFloatKey::new(Node::BoolAnd, vec![value.clone(), value]);
        }
        let bytes = value.canonical_bytes(1001, 11000).unwrap();
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 1001);
        assert!(bytes.len() < 11000);
        assert_eq!(
            value.canonical_bytes(1000, 11000),
            Err(WeakFloatEncodingError::NodeLimit)
        );
    }
}
