//! Versioned, exact semantic DAG encoding for immutable replay facts.
use super::*;
use crate::retained_metadata::{EvalRetainedMetadataError, admit, push, reserve};

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
impl std::error::Error for WeakFloatEncodingError {
}

struct Output<'a, E> {
    bytes: Vec<u8>,
    limit: usize,
    charge: &'a mut dyn FnMut(usize, usize) -> Result<(), E>,
}
impl<E> Output<'_, E> {
    fn append(&mut self, bytes: &[u8]) -> Result<(), WeakFloatAdmissionError<E>> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(WeakFloatAdmissionError::Encoding(
                WeakFloatEncodingError::ByteLimit,
            ));
        }
        admit(bytes.len().saturating_add(1), 0, self.charge)?;
        reserve(&mut self.bytes, bytes.len(), self.charge)?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn byte(&mut self, value: u8) -> Result<(), WeakFloatAdmissionError<E>> {
        self.append(&[value])
    }
    fn integer_type(&mut self, ty: IntegerType) -> Result<(), WeakFloatAdmissionError<E>> {
        self.append(&[ty.bits() as u8, u8::from(ty.signed())])
    }
    fn text(&mut self, text: &str) -> Result<(), WeakFloatAdmissionError<E>> {
        let length = u32::try_from(text.len())
            .map_err(|_| WeakFloatAdmissionError::Encoding(WeakFloatEncodingError::ByteLimit))?;
        self.append(&length.to_le_bytes())?;
        self.append(text.as_bytes())
    }
    fn tag(&mut self, node: &Node) -> Result<(), WeakFloatAdmissionError<E>> {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WeakFloatAdmissionError<E> {
    Encoding(WeakFloatEncodingError),
    Admission(E),
    CapacityOverflow,
    Allocation,
}
impl<E> From<EvalRetainedMetadataError<E>> for WeakFloatAdmissionError<E> {
    fn from(error: EvalRetainedMetadataError<E>) -> Self {
        match error {
            EvalRetainedMetadataError::Admission(error) => Self::Admission(error),
            EvalRetainedMetadataError::CapacityOverflow => Self::CapacityOverflow,
            EvalRetainedMetadataError::Allocation => Self::Allocation,
        }
    }
}
fn resolved_id<E>(
    resolved: &[(*const KeyNode, u32)],
    pointer: *const KeyNode,
    charge: &mut dyn FnMut(usize, usize) -> Result<(), E>,
) -> Result<Option<u32>, WeakFloatAdmissionError<E>> {
    for (existing, id) in resolved {
        admit(1, 0, charge)?;
        if *existing == pointer {
            return Ok(Some(*id));
        }
    }
    Ok(None)
}
impl WeakFloatKey {
    /// Export exact semantic identity through the same engine as metered callers.
    pub fn canonical_bytes(
        &self,
        max_nodes: usize,
        max_bytes: usize,
    ) -> Result<Vec<u8>, WeakFloatEncodingError> {
        match self.canonical_bytes_with_work(max_nodes, max_bytes, &mut |_, _| {
            Ok::<_, std::convert::Infallible>(())
        }) {
            Ok(bytes) => Ok(bytes),
            Err(WeakFloatAdmissionError::Encoding(error)) => Err(error),
            Err(WeakFloatAdmissionError::Admission(never)) => match never {},
            Err(
                WeakFloatAdmissionError::CapacityOverflow | WeakFloatAdmissionError::Allocation,
            ) => Err(WeakFloatEncodingError::ByteLimit),
        }
    }
    /// Debit each real traversal, comparison and old/new scratch allocation
    /// BEFORE inspecting/copying it. Nodes and decimal signatures stay borrowed.
    /// Equal keys produce exactly the same version-one records as the unmetered API.
    pub fn canonical_bytes_with_work<E>(
        &self,
        max_nodes: usize,
        max_bytes: usize,
        charge: &mut impl FnMut(usize, usize) -> Result<(), E>,
    ) -> Result<Vec<u8>, WeakFloatAdmissionError<E>> {
        admit(
            1,
            std::mem::size_of::<Vec<(&Node, [u32; 3], u8)>>()
                + std::mem::size_of::<Vec<(*const KeyNode, u32)>>()
                + std::mem::size_of::<Vec<(&WeakFloatKey, bool)>>()
                + std::mem::size_of::<Vec<u8>>(),
            charge,
        )?;
        let mut output = Output {
            bytes: Vec::new(),
            limit: max_bytes,
            charge,
        };
        output.append(b"WFK\x01\0\0\0\0")?;
        let mut resolved = Vec::new();
        let mut canonical: Vec<(&Node, [u32; 3], u8)> = Vec::new();
        let mut pending = Vec::new();
        push(&mut pending, (self, false), output.charge)?;
        let mut visited = 0usize;
        while let Some((key, finish)) = pending.pop() {
            admit(1, 0, output.charge)?;
            let pointer = std::sync::Arc::as_ptr(&key.0);
            if resolved_id(&resolved, pointer, output.charge)?.is_some() {
                continue;
            }
            if !finish {
                if visited >= max_nodes {
                    return Err(WeakFloatAdmissionError::Encoding(
                        WeakFloatEncodingError::NodeLimit,
                    ));
                }
                visited += 1;
                push(&mut pending, (key, true), output.charge)?;
                for child in key.0.children.iter().rev() {
                    admit(1, 0, output.charge)?;
                    push(&mut pending, (child, false), output.charge)?;
                }
                continue;
            }
            if key.0.children.len() > 3 {
                return Err(WeakFloatAdmissionError::Encoding(
                    WeakFloatEncodingError::NodeLimit,
                ));
            }
            let mut children = [0; 3];
            for (index, child) in key.0.children.iter().enumerate() {
                admit(1, 0, output.charge)?;
                children[index] =
                    resolved_id(&resolved, std::sync::Arc::as_ptr(&child.0), output.charge)?
                        .ok_or(WeakFloatAdmissionError::Encoding(
                            WeakFloatEncodingError::NodeLimit,
                        ))?;
            }
            let count = key.0.children.len() as u8;
            if let Node::Decimal(value) = &key.0.tag
                && value.spelling().len() > max_bytes
            {
                return Err(WeakFloatAdmissionError::Encoding(
                    WeakFloatEncodingError::ByteLimit,
                ));
            }
            let mut existing = None;
            for (index, (node, previous, length)) in canonical.iter().enumerate() {
                let spelling_work = match (*node, &key.0.tag) {
                    (Node::Decimal(left), Node::Decimal(right)) => {
                        left.spelling().len().saturating_add(right.spelling().len())
                    }
                    _ => 0,
                };
                admit(spelling_work.saturating_add(4), 0, output.charge)?;
                if *length == count && *previous == children && **node == key.0.tag {
                    existing = Some(index as u32);
                    break;
                }
            }
            let id = match existing {
                Some(id) => id,
                None => {
                    let id = u32::try_from(canonical.len()).map_err(|_| {
                        WeakFloatAdmissionError::Encoding(WeakFloatEncodingError::NodeLimit)
                    })?;
                    output.tag(&key.0.tag)?;
                    output.byte(count)?;
                    for child in &children[..usize::from(count)] {
                        output.append(&child.to_le_bytes())?;
                    }
                    push(&mut canonical, (&key.0.tag, children, count), output.charge)?;
                    id
                }
            };
            push(&mut resolved, (pointer, id), output.charge)?;
        }
        let count = u32::try_from(canonical.len())
            .map_err(|_| WeakFloatAdmissionError::Encoding(WeakFloatEncodingError::NodeLimit))?;
        admit(1, 0, output.charge)?;
        output.bytes[4..8].copy_from_slice(&count.to_le_bytes());
        let root = resolved_id(&resolved, std::sync::Arc::as_ptr(&self.0), output.charge)?.ok_or(
            WeakFloatAdmissionError::Encoding(WeakFloatEncodingError::NodeLimit),
        )?;
        output.append(&root.to_le_bytes())?;
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
