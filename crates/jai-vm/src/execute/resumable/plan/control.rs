//! Control flow contains private IDs, never suspended borrowed statement trees.
use super::*;

#[cfg(test)]
mod tests;

impl Builder<'_> {
    pub(super) fn block(&mut self, block: &Block, depth: usize) -> Result<BlockId, Error> {
        self.reserve(1, depth)?;
        self.reserve(block.statements.len(), depth)?;
        let mut statements = Vec::with_capacity(block.statements.len());
        for statement in &block.statements {
            statements.push(self.statement(statement, depth + 1)?);
        }
        let id = BlockId(self.blocks.len());
        self.blocks.push(BlockCode {
            statements: statements.into(),
            flow: block.flow,
        });
        Ok(id)
    }

    fn subject(&mut self, statement: &Statement, depth: usize) -> Result<BlockId, Error> {
        self.reserve(2, depth)?;
        let statement = self.statement(statement, depth + 1)?;
        let id = BlockId(self.blocks.len());
        self.blocks.push(BlockCode {
            statements: vec![statement].into(),
            flow: Flow::FallsThrough,
        });
        Ok(id)
    }

    fn destinations(
        &mut self,
        places: &[Option<Place>],
        depth: usize,
    ) -> Result<Box<[Option<NodeId>]>, Error> {
        self.reserve(places.len(), depth)?;
        let mut nodes = Vec::with_capacity(places.len());
        for place in places {
            nodes.push(
                place
                    .as_ref()
                    .map(|place| self.place(*place, depth + 1))
                    .transpose()?,
            );
        }
        Ok(nodes.into())
    }

    fn statement(&mut self, statement: &Statement, depth: usize) -> Result<StatementCode, Error> {
        self.reserve(0, depth)?;
        Ok(match statement {
            Statement::Store(destination, value) => StatementCode::Store {
                destination: self.place(*destination, depth + 1)?,
                value: self.value(value, depth + 1)?,
            },
            Statement::StoreInt(destination, value) => StatementCode::Store {
                destination: self.place(destination.place(), depth + 1)?,
                value: self.int(value, depth + 1)?,
            },
            Statement::StoreBool(destination, value) => StatementCode::Store {
                destination: self.place(destination.place(), depth + 1)?,
                value: self.bool(value, depth + 1)?,
            },
            Statement::DiscardValue(value) => StatementCode::Discard(self.value(value, depth + 1)?),
            Statement::DiscardInt(value) => StatementCode::Discard(self.int(value, depth + 1)?),
            Statement::DiscardBool(value) => StatementCode::Discard(self.bool(value, depth + 1)?),
            Statement::CallVoid(call) => {
                StatementCode::Discard(self.call(call, None, depth + 1)?)
            }
            Statement::CallResults { call, destinations } => {
                let destinations = self.destinations(destinations, depth + 1)?;
                let call = self.call(call, None, depth + 1)?;
                StatementCode::CallResults { call, destinations }
            }
            Statement::IndirectCallResults {
                callee,
                arguments,
                destinations,
                ..
            } => {
                let destinations = self.destinations(destinations, depth + 1)?;
                let call = self.indirect_call(callee, arguments, None, depth + 1)?;
                StatementCode::CallResults { call, destinations }
            }
            Statement::Exit(exit) => StatementCode::Exit(self.exit(exit, depth + 1)?),
            Statement::Cleanup(id) => StatementCode::Cleanup(*id),
            Statement::If(condition, then_block, else_block) => StatementCode::If {
                condition: self.bool(condition, depth + 1)?,
                then_block: self.block(then_block, depth + 1)?,
                else_block: self.block(else_block, depth + 1)?,
            },
            Statement::While {
                id,
                condition,
                body,
            } => StatementCode::While {
                id: *id,
                condition: self.condition(condition, depth + 1)?,
                body: self.block(body, depth + 1)?,
            },
            Statement::Range(range) => StatementCode::Range(RangeCode {
                id: range.id,
                iterator: self.place(range.iterator.local().place(), depth + 1)?,
                integer_type: range.iterator.ty(),
                start: self.int(&range.start, depth + 1)?,
                end: self.int(&range.end, depth + 1)?,
                direction: range.direction,
                body: self.block(&range.body, depth + 1)?,
            }),
            Statement::Cases(cases) => StatementCode::Cases(self.cases(cases, depth + 1)?),
            Statement::Block(block) => StatementCode::Block(self.block(block, depth + 1)?),
            Statement::PushContext { id, value, body } => StatementCode::PushContext {
                id: *id,
                value: self.value(value, depth + 1)?,
                body: self.block(body, depth + 1)?,
            },
            Statement::Simd(block) => StatementCode::Simd(self.simd(block, depth + 1)?),
        })
    }

    fn exit(&mut self, exit: &Exit, depth: usize) -> Result<ExitCode, Error> {
        self.reserve(exit.cleanups.len(), depth)?;
        let (values, transfer) = match &exit.transfer {
            Transfer::ReturnValues(values) => {
                self.reserve(values.len(), depth)?;
                let mut nodes = Vec::with_capacity(values.len());
                for value in values {
                    nodes.push(self.value(value, depth + 1)?);
                }
                (nodes, TransferCode::Return)
            }
            Transfer::ReturnInt(value) => {
                self.reserve(1, depth)?;
                (vec![self.int(value, depth + 1)?], TransferCode::Return)
            }
            Transfer::ReturnBool(value) => {
                self.reserve(1, depth)?;
                (vec![self.bool(value, depth + 1)?], TransferCode::Return)
            }
            Transfer::ReturnVoid => (vec![], TransferCode::Return),
            Transfer::Break(id) => (vec![], TransferCode::Break(*id)),
            Transfer::Continue(id) => (vec![], TransferCode::Continue(*id)),
        };
        Ok(ExitCode {
            values: values.into(),
            cleanups: exit.cleanups.clone().into(),
            transfer,
        })
    }

    fn condition(
        &mut self,
        condition: &LoopCondition,
        depth: usize,
    ) -> Result<ConditionCode, Error> {
        self.reserve(0, depth)?;
        Ok(match condition {
            LoopCondition::Value(value) => ConditionCode::Value(self.bool(value, depth + 1)?),
            LoopCondition::BoundInt(destination, value) => ConditionCode::BoundInt {
                destination: self.place(destination.local().place(), depth + 1)?,
                value: self.int(value, depth + 1)?,
            },
            LoopCondition::BoundBool(destination, value) => ConditionCode::BoundBool {
                destination: self.place(destination.local().place(), depth + 1)?,
                value: self.bool(value, depth + 1)?,
            },
        })
    }

    fn cases(&mut self, cases: &Cases, depth: usize) -> Result<CasesCode, Error> {
        self.reserve(cases.arms.len(), depth)?;
        let subject = self.subject(&cases.subject, depth + 1)?;
        let mut arms = Vec::with_capacity(cases.arms.len());
        for arm in &cases.arms {
            arms.push(CaseCode {
                condition: self.bool(&arm.condition, depth + 1)?,
                body: self.block(&arm.body, depth + 1)?,
                through: arm.through,
            });
        }
        let default = cases
            .default
            .as_ref()
            .map(|block| self.block(block, depth + 1))
            .transpose()?;
        Ok(CasesCode {
            subject,
            arms: arms.into(),
            default,
            flow: cases.flow,
            exhaustive: cases.exhaustive,
        })
    }

    fn simd(&mut self, block: &SimdBlock, depth: usize) -> Result<SimdCode, Error> {
        self.reserve(block.registers().len(), depth)?;
        self.reserve(block.instructions().len(), depth)?;
        let registers = block.registers().to_vec().into_boxed_slice();
        let mut instructions = Vec::with_capacity(block.instructions().len());
        for instruction in block.instructions() {
            let width = |register| {
                block
                    .width(register)
                    .map_err(|error| Error::IrValidation(error.to_string()))
            };
            instructions.push(match instruction {
                SimdInstruction::DebugTrap | SimdInstruction::Arm64DebugTrap => {
                    SimdInstructionCode::Trap
                }
                SimdInstruction::Load {
                    destination,
                    interpretation,
                    address,
                } => SimdInstructionCode::Load {
                    destination: destination.index(),
                    interpretation: *interpretation,
                    address: self.value(address, depth + 1)?,
                    width: width(*destination)?,
                },
                SimdInstruction::Store {
                    source,
                    interpretation,
                    address,
                } => SimdInstructionCode::Store {
                    source: source.index(),
                    interpretation: *interpretation,
                    address: self.value(address, depth + 1)?,
                    width: width(*source)?,
                },
                SimdInstruction::Add {
                    destination,
                    left,
                    right,
                    interpretation,
                } => SimdInstructionCode::Add {
                    destination: destination.index(),
                    left: left.index(),
                    right: right.index(),
                    interpretation: *interpretation,
                    width: width(*destination)?,
                },
            });
        }
        Ok(SimdCode {
            features: block.features(),
            registers,
            instructions: instructions.into(),
        })
    }
}
