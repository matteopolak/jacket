//! Typed positions in the final IR, resolved without recursive traversal.
use crate::{Block, CleanupId, IrError, Procedure, ProcedureId, Statement};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockRoot {
    ProcedureBody(ProcedureId),
    Cleanup {
        procedure: ProcedureId,
        cleanup: CleanupId,
    },
}
impl BlockRoot {
    pub fn procedure(self) -> ProcedureId {
        match self {
            Self::ProcedureBody(procedure)
            | Self::Cleanup {
                procedure, ..
            } => procedure,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DebugBranch {
    Block,
    IfThen,
    IfElse,
    While,
    Range,
    PushContext,
    CaseArm(usize),
    CaseDefault,
    CaseSubject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DebugPathStep {
    Statement(usize),
    Child(DebugBranch),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockPath {
    pub root: BlockRoot,
    pub steps: Box<[DebugPathStep]>,
}
impl BlockPath {
    pub fn new(root: BlockRoot) -> Self {
        Self {
            root,
            steps: Box::new([]),
        }
    }
    pub fn procedure(id: ProcedureId) -> Self {
        Self::new(BlockRoot::ProcedureBody(id))
    }
    pub fn cleanup(procedure: ProcedureId, cleanup: CleanupId) -> Self {
        Self::new(BlockRoot::Cleanup {
            procedure,
            cleanup,
        })
    }
    pub fn child(&self, statement_index: usize, branch: DebugBranch) -> Self {
        self.then([
            DebugPathStep::Statement(statement_index),
            DebugPathStep::Child(branch),
        ])
    }
    pub fn statement(&self, index: usize) -> StatementPath {
        StatementPath::in_block(self, index)
    }
    pub fn then(&self, steps: impl IntoIterator<Item = DebugPathStep>) -> Self {
        Self {
            root: self.root,
            steps: self.steps.iter().copied().chain(steps).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StatementPath {
    pub root: BlockRoot,
    pub steps: Box<[DebugPathStep]>,
}
impl StatementPath {
    pub fn in_block(block: &BlockPath, index: usize) -> Self {
        Self {
            root: block.root,
            steps: block
                .steps
                .iter()
                .copied()
                .chain([DebugPathStep::Statement(index)])
                .collect(),
        }
    }
    pub fn child(&self, branch: DebugBranch) -> Self {
        Self {
            root: self.root,
            steps: self
                .steps
                .iter()
                .copied()
                .chain([DebugPathStep::Child(branch)])
                .collect(),
        }
    }
    pub fn block_child(&self, branch: DebugBranch) -> BlockPath {
        BlockPath {
            root: self.root,
            steps: self
                .steps
                .iter()
                .copied()
                .chain([DebugPathStep::Child(branch)])
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LocalDeclaration {
    Parameter(usize),
    Statement(StatementPath),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalSource {
    pub name: String,
    pub location: super::DebugSourceLocation,
    pub scope: BlockPath,
    pub declaration: LocalDeclaration,
}

enum Node<'a> {
    Block(&'a Block),
    Statement(&'a Statement),
}

fn invalid(kind: &'static str, index: usize) -> IrError {
    IrError::UnknownIdentity {
        kind,
        index,
    }
}

fn resolve<'a>(
    root: BlockRoot,
    steps: &[DebugPathStep],
    procedure: &'a Procedure,
) -> Result<Node<'a>, IrError> {
    if root.procedure() != procedure.id {
        return Err(invalid("debug path procedure", root.procedure().index()));
    }
    if steps.len() > 512 {
        return Err(IrError::VerificationDepth);
    }
    let mut node = Node::Block(match root {
        BlockRoot::ProcedureBody(_) => &procedure.body,
        BlockRoot::Cleanup {
            cleanup, ..
        } => {
            &procedure
                .cleanups
                .get(cleanup.index())
                .ok_or_else(|| invalid("debug cleanup", cleanup.index()))?
                .body
        }
    });
    for (position, step) in steps.iter().enumerate() {
        node = match (node, step) {
            (Node::Block(block), DebugPathStep::Statement(index)) => Node::Statement(
                block
                    .statements
                    .get(*index)
                    .ok_or_else(|| invalid("debug statement", *index))?,
            ),
            (Node::Statement(statement), DebugPathStep::Child(branch)) => match (statement, branch)
            {
                (Statement::Block(block), DebugBranch::Block)
                | (Statement::If(_, block, _), DebugBranch::IfThen)
                | (Statement::If(_, _, block), DebugBranch::IfElse)
                | (
                    Statement::While {
                        body: block, ..
                    },
                    DebugBranch::While,
                )
                | (
                    Statement::PushContext {
                        body: block, ..
                    },
                    DebugBranch::PushContext,
                ) => Node::Block(block),
                (Statement::Range(range), DebugBranch::Range) => Node::Block(&range.body),
                (Statement::Cases(cases), DebugBranch::CaseArm(index)) => Node::Block(
                    &cases
                        .arms
                        .get(*index)
                        .ok_or_else(|| invalid("debug case arm", *index))?
                        .body,
                ),
                (Statement::Cases(cases), DebugBranch::CaseDefault) => Node::Block(
                    cases
                        .default
                        .as_ref()
                        .ok_or_else(|| invalid("debug case default", position))?,
                ),
                (Statement::Cases(cases), DebugBranch::CaseSubject) => {
                    Node::Statement(&cases.subject)
                }
                _ => return Err(invalid("debug branch", position)),
            },
            _ => return Err(invalid("debug path step", position)),
        };
    }
    Ok(node)
}

pub(super) fn block<'a>(path: &BlockPath, procedure: &'a Procedure) -> Result<&'a Block, IrError> {
    match resolve(path.root, &path.steps, procedure)? {
        Node::Block(block) => Ok(block),
        Node::Statement(_) => Err(invalid("debug block path", path.steps.len())),
    }
}

pub(super) fn statement<'a>(
    path: &StatementPath,
    procedure: &'a Procedure,
) -> Result<&'a Statement, IrError> {
    match resolve(path.root, &path.steps, procedure)? {
        Node::Statement(statement) => Ok(statement),
        Node::Block(_) => Err(invalid("debug statement path", path.steps.len())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BoolExpr, CaseArm, Cases, Flow, LoopCondition, LoopId, PushContextId, RangeLoop, ValueExpr,
    };
    use jai_types::{
        CallingConvention, ContextMode, Direction, Integer, IntegerType, ScalarType, TypeRegistry,
        Variadic,
    };

    fn leaf() -> Block {
        Block {
            statements: vec![Statement::DiscardBool(BoolExpr::Constant(true))],
            flow: Flow::FallsThrough,
        }
    }

    fn fixture() -> Procedure {
        let mut types = TypeRegistry::new();
        let signature = types
            .procedure(jai_types::ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: Variadic::None,
            })
            .unwrap();
        let id = ProcedureId::new(7);
        let iterator = crate::Local::new(id, 0, ScalarType::Int(IntegerType::S64), &types)
            .integer(&types)
            .unwrap();
        let int = || crate::IntExpr::constant(Integer::checked(IntegerType::S64, 0).unwrap());
        Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![],
            body: Block {
                statements: vec![
                    Statement::Block(leaf()),
                    Statement::If(BoolExpr::Constant(true), leaf(), leaf()),
                    Statement::While {
                        id: LoopId::new(0),
                        condition: LoopCondition::Value(BoolExpr::Constant(true)),
                        body: leaf(),
                    },
                    Statement::Range(RangeLoop {
                        id: LoopId::new(1),
                        iterator,
                        start: int(),
                        end: int(),
                        direction: Direction::Forward,
                        body: leaf(),
                    }),
                    Statement::PushContext {
                        id: PushContextId::new(id, 0),
                        value: ValueExpr::Bool(BoolExpr::Constant(true)),
                        body: leaf(),
                    },
                    Statement::Cases(Cases {
                        subject: Box::new(Statement::Block(leaf())),
                        arms: vec![CaseArm {
                            condition: BoolExpr::Constant(true),
                            body: leaf(),
                            through: false,
                        }],
                        default: Some(leaf()),
                        flow: Flow::FallsThrough,
                        exhaustive: false,
                    }),
                ],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![leaf().into()],
        }
    }

    #[test]
    fn root_and_cleanup_paths_resolve_the_actual_blocks() {
        let procedure = fixture();
        assert!(std::ptr::eq(
            block(&BlockPath::procedure(procedure.id), &procedure).unwrap(),
            &procedure.body
        ));
        let path = BlockPath::cleanup(procedure.id, CleanupId::new(0));
        assert!(std::ptr::eq(
            block(&path, &procedure).unwrap(),
            &procedure.cleanups[0].body
        ));
        assert_eq!(path.root.procedure(), procedure.id);
    }

    #[test]
    fn every_block_branch_and_its_statement_resolves() {
        let procedure = fixture();
        let root = BlockPath::procedure(procedure.id);
        for (index, branch) in [
            (0, DebugBranch::Block),
            (1, DebugBranch::IfThen),
            (1, DebugBranch::IfElse),
            (2, DebugBranch::While),
            (3, DebugBranch::Range),
            (4, DebugBranch::PushContext),
            (5, DebugBranch::CaseArm(0)),
            (5, DebugBranch::CaseDefault),
        ] {
            let path = root.child(index, branch);
            let body = block(&path, &procedure).unwrap();
            let statement_path = path.statement(0);
            assert!(std::ptr::eq(
                statement(&statement_path, &procedure).unwrap(),
                &body.statements[0]
            ));
        }
    }

    #[test]
    fn case_subject_is_a_statement_and_can_have_a_child_block() {
        let procedure = fixture();
        let root = BlockPath::procedure(procedure.id);
        let subject = StatementPath::in_block(&root, 5).child(DebugBranch::CaseSubject);
        assert!(matches!(
            statement(&subject, &procedure),
            Ok(Statement::Block(_))
        ));
        let path = subject.block_child(DebugBranch::Block);
        assert!(block(&path, &procedure).is_ok());
        assert!(block(&root.child(5, DebugBranch::CaseSubject), &procedure).is_err());
    }

    #[test]
    fn resolution_rejects_wrong_owner_and_missing_indices() {
        let procedure = fixture();
        assert!(block(&BlockPath::procedure(ProcedureId::new(8)), &procedure).is_err());
        assert!(
            block(
                &BlockPath::cleanup(procedure.id, CleanupId::new(1)),
                &procedure
            )
            .is_err()
        );
        let root = BlockPath::procedure(procedure.id);
        assert!(statement(&StatementPath::in_block(&root, 6), &procedure).is_err());
        assert!(block(&root.child(5, DebugBranch::CaseArm(1)), &procedure).is_err());
    }

    #[test]
    fn resolution_rejects_wrong_step_branch_and_final_node_kind() {
        let procedure = fixture();
        let root = BlockPath::procedure(procedure.id);
        assert!(
            block(
                &root.then([DebugPathStep::Child(DebugBranch::Block)]),
                &procedure
            )
            .is_err()
        );
        assert!(block(&root.child(0, DebugBranch::IfThen), &procedure).is_err());
        assert!(block(&root.then([DebugPathStep::Statement(0)]), &procedure).is_err());
        let path = StatementPath {
            root: root.root,
            steps: Box::new([]),
        };
        assert!(statement(&path, &procedure).is_err());
    }

    #[test]
    fn resolution_rejects_missing_case_default() {
        let mut procedure = fixture();
        let Statement::Cases(cases) = &mut procedure.body.statements[5] else {
            panic!()
        };
        cases.default = None;
        let path = BlockPath::procedure(procedure.id).child(5, DebugBranch::CaseDefault);
        assert!(block(&path, &procedure).is_err());
    }

    #[test]
    fn resolution_rejects_more_than_512_steps_before_traversal() {
        let procedure = fixture();
        let path = BlockPath::procedure(procedure.id)
            .then(std::iter::repeat_n(DebugPathStep::Statement(0), 513));
        assert!(matches!(
            block(&path, &procedure),
            Err(IrError::VerificationDepth)
        ));
    }

    #[test]
    fn resolution_accepts_exactly_512_valid_steps() {
        let mut procedure = fixture();
        let mut body = leaf();
        for _ in 0..256 {
            body = Block {
                statements: vec![Statement::Block(body)],
                flow: Flow::FallsThrough,
            };
        }
        procedure.body = body;
        let path = BlockPath::procedure(procedure.id).then(
            std::iter::repeat_n(
                [
                    DebugPathStep::Statement(0),
                    DebugPathStep::Child(DebugBranch::Block),
                ],
                256,
            )
            .flatten(),
        );
        assert_eq!(path.steps.len(), 512);
        assert!(matches!(
            block(&path, &procedure).unwrap().statements[0],
            Statement::DiscardBool(_)
        ));
    }
}
