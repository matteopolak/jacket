//! Prepare demanded storage before an action consumes its captured operands.
use super::*;

impl Machine {
    pub(super) fn prepare_task<P: ProcedureProvider + ?Sized, E: CompilerEffects>(
        &self,
        vm: &mut Vm<'_, P, E>,
        task: &Task,
    ) -> Result<()> {
        match &task.action {
            Action::Eval(node) => {
                if let Some(code) = &task.code
                    && let NodeKind::SequencePack {
                        ty, ..
                    } = &code.nodes[node.index()].kind
                {
                    let TypeKind::Slice(element) = vm.provider.types().kind(*ty)? else {
                        return Err(Error::InvalidIr("sequence pack requires slice type").into());
                    };
                    vm.prepare_layout(*element)?;
                }
            }
            Action::RecordStart {
                node,
            }
            | Action::RecordNext {
                node, ..
            } => {
                let code = task
                    .code
                    .as_ref()
                    .ok_or(Error::InvalidIr("record action has no plan"))?;
                let NodeKind::RecordBuild {
                    ty, ..
                } = &code.nodes[node.index()].kind
                else {
                    return Err(Error::InvalidIr("record action has wrong plan node").into());
                };
                vm.prepare_layout(*ty)?;
            }
            Action::OrderedRecordStart {
                node,
            } => {
                let code = task
                    .code
                    .as_ref()
                    .ok_or(Error::InvalidIr("ordered record action has no plan"))?;
                let NodeKind::OrderedRecord {
                    ty,
                    initializers,
                    ..
                } = &code.nodes[node.index()].kind
                else {
                    return Err(
                        Error::InvalidIr("ordered record action has wrong plan node").into(),
                    );
                };
                vm.prepare_ordered_record(*ty, initializers.iter().map(|(path, _)| path.as_ref()))?;
            }
            Action::IndexBase {
                node,
            } => {
                let code = task
                    .code
                    .as_ref()
                    .ok_or(Error::InvalidIr("index action has no plan"))?;
                let NodeKind::IndexPlace {
                    base_type, ..
                } = &code.nodes[node.index()].kind
                else {
                    return Err(Error::InvalidIr("index action has wrong plan node").into());
                };
                if !matches!(
                    vm.provider.types().kind(*base_type)?,
                    TypeKind::FixedArray { .. }
                ) {
                    vm.prepare_pointer_layouts(self.place_at(0)?, true)?;
                }
            }
            Action::IndexFinish {
                pointer,
                snapshot,
                ..
            } => {
                let pointer = match snapshot {
                    Some(
                        Value::Slice {
                            pointer, ..
                        }
                        | Value::DynamicArray {
                            pointer, ..
                        }
                        | Value::StringView {
                            pointer, ..
                        }
                        | Value::Pointer(pointer),
                    ) => pointer,
                    _ => pointer,
                };
                // Bounds/null errors remain in the index operation itself.
                // Null address readiness must not demand storage or consume the index.
                if !pointer.is_null() {
                    vm.prepare_pointer_layouts(pointer, true)?;
                }
            }
            Action::PushStart {
                ..
            } => {
                let ty = vm
                    .provider
                    .context()
                    .ok_or(Error::InvalidIr("push context schema missing"))?
                    .record_type;
                vm.prepare_layout(ty)?;
            }
            Action::Store => vm.prepare_pointer_layouts(self.place_at(1)?, true)?,
            Action::StoreResults(destinations) => {
                for pointer in destinations.iter().flatten() {
                    vm.prepare_pointer_layouts(pointer, true)?;
                }
            }
            Action::RangeSetup {
                ..
            } => vm.prepare_pointer_layouts(self.place_at(0)?, true)?,
            Action::RangeNext(range) => {
                vm.prepare_pointer_layouts(
                    range
                        .pointer
                        .as_ref()
                        .ok_or(Error::InvalidIr("range has no iterator storage"))?,
                    true,
                )?;
            }
            Action::PackNext {
                node,
                index,
                ..
            } if *index != 0 => {
                let code = task
                    .code
                    .as_ref()
                    .ok_or(Error::InvalidIr("pack action has no plan"))?;
                let NodeKind::SequencePack {
                    parts, ..
                } = &code.nodes[node.index()].kind
                else {
                    return Err(Error::InvalidIr("pack action has wrong plan node").into());
                };
                let operand = self
                    .operands
                    .last()
                    .ok_or(Error::InvalidIr("pack has no operand"))?;
                match (parts[index - 1].mode, operand) {
                    (PackPartMode::ElementPlace, Operand::Place(pointer)) => {
                        vm.prepare_pointer_layouts(pointer, true)?;
                    }
                    (
                        PackPartMode::Spread,
                        Operand::Value(Value::Slice {
                            pointer,
                            count,
                            ..
                        }),
                    ) if *count > 0 => vm.prepare_pointer_layouts(pointer, true)?,
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn place_at(&self, from_top: usize) -> Result<&Pointer> {
        let index = self
            .operands
            .len()
            .checked_sub(from_top + 1)
            .ok_or(Error::InvalidIr("missing continuation place operand"))?;
        let Operand::Place(pointer) = &self.operands[index] else {
            return Err(Error::InvalidIr("continuation operand is not a place").into());
        };
        Ok(pointer)
    }
}
