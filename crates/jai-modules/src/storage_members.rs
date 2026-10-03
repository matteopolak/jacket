//! Canonical source paths into real globals; semantic hydration owns field IDs.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

pub const MAX_SOURCE_STORAGE_PATH_DEPTH: usize = 128;

static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceStorageMemberId {
    session: u64,
    index: usize,
}
impl SourceStorageMemberId {
    pub fn index(self) -> usize {
        self.index
    }
}
#[derive(Clone, Debug)]
pub struct SourceStorageMember {
    owner: DeclarationId,
    path: Box<[Symbol]>,
    location: SourceSpan,
}
impl SourceStorageMember {
    pub fn owner(&self) -> DeclarationId {
        self.owner
    }
    pub fn path(&self) -> &[Symbol] {
        &self.path
    }
    pub fn location(&self) -> SourceSpan {
        self.location
    }
}
#[derive(Clone, Debug)]
pub(super) struct StorageMembers {
    session: u64,
    records: Vec<SourceStorageMember>,
}
impl Default for StorageMembers {
    fn default() -> Self {
        Self {
            session: NEXT_SESSION.fetch_add(1, Ordering::Relaxed),
            records: Vec::new(),
        }
    }
}
impl StorageMembers {
    pub(super) fn len(&self) -> usize {
        self.records.len()
    }
    pub(super) fn truncate(&mut self, len: usize) {
        self.records.truncate(len);
    }
    pub(super) fn intern(
        &mut self,
        owner: DeclarationId,
        path: &[Symbol],
        location: SourceSpan,
    ) -> SourceStorageMemberId {
        let index = self
            .records
            .iter()
            .position(|record| record.owner == owner && record.path.as_ref() == path)
            .unwrap_or_else(|| {
                let index = self.records.len();
                self.records.push(SourceStorageMember {
                    owner,
                    path: path.into(),
                    location,
                });
                index
            });
        SourceStorageMemberId {
            session: self.session,
            index,
        }
    }
    fn get(&self, id: SourceStorageMemberId) -> Option<&SourceStorageMember> {
        (id.session == self.session)
            .then(|| self.records.get(id.index))
            .flatten()
    }
}
impl ModuleGraph {
    pub fn source_storage_member(&self, id: SourceStorageMemberId) -> Option<&SourceStorageMember> {
        self.storage_members.get(id)
    }
    pub fn source_storage_member_id(
        &self,
        owner: DeclarationId,
        path: &[Symbol],
    ) -> Option<SourceStorageMemberId> {
        self.storage_members
            .records
            .iter()
            .position(|record| record.owner == owner && record.path.as_ref() == path)
            .map(|index| SourceStorageMemberId {
                session: self.storage_members.session,
                index,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn overlay(source: &str) -> SourceOverlay {
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(
                Path::new("/storage-member/main.jai"),
                source.as_bytes().to_vec(),
            )
            .unwrap();
        overlay
    }
    #[test]
    fn publication_keeps_full_original_global_path_and_session_identity() {
        let source = overlay(
            "Inner::struct{value:int;} Outer::struct{inner:Inner;} state:Outer; unrelated:Outer; using state.inner; main::(){}",
        );
        let mut discovery = GraphDiscovery::new(
            Path::new("/storage-member/main.jai"),
            GraphOptions::default(),
            &source,
        )
        .unwrap();
        assert!(!discovery.advance().unwrap().is_complete());
        let request = discovery.pending_using_requests().pop().unwrap();
        let owner = discovery
            .graph()
            .declarations()
            .iter()
            .find(|decl| discovery.graph().symbols().name(decl.name()) == "state")
            .unwrap()
            .id();
        let fields = [
            discovery.graph().symbols().find("inner").unwrap(),
            discovery.graph().symbols().find("value").unwrap(),
        ];
        let invalid = FileUsingDecision {
            storage_members: vec![UsingStorageMember {
                owner,
                path: vec![],
                destination: "value".into(),
            }],
            ..Default::default()
        };
        assert!(matches!(
            discovery.resolve_using(request.id, invalid),
            Err(GraphError::InvalidUsingResponse(
                UsingResponseError::InvalidSourceIdentity
            ))
        ));
        let unrelated = discovery
            .graph()
            .declarations()
            .iter()
            .find(|decl| discovery.graph().symbols().name(decl.name()) == "unrelated")
            .unwrap()
            .id();
        let redirected = FileUsingDecision {
            storage_members: vec![UsingStorageMember {
                owner: unrelated,
                path: fields.to_vec(),
                destination: "value".into(),
            }],
            ..Default::default()
        };
        assert!(matches!(
            discovery.resolve_using(request.id, redirected),
            Err(GraphError::InvalidUsingResponse(
                UsingResponseError::InvalidSourceIdentity
            ))
        ));
        let decision = FileUsingDecision {
            storage_members: vec![UsingStorageMember {
                owner,
                path: fields.to_vec(),
                destination: "value".into(),
            }],
            ..Default::default()
        };
        discovery.resolve_using(request.id, decision).unwrap();
        let name = discovery.graph().symbols().find("value").unwrap();
        let Binding::StorageMember(id) = discovery
            .graph()
            .lookup(
                request.file,
                &NamePath {
                    root: name,
                    members: vec![],
                },
            )
            .unwrap()
        else {
            panic!("expected real storage path binding")
        };
        let record = discovery.graph().source_storage_member(id).unwrap();
        assert_eq!(record.owner(), owner);
        assert_eq!(record.path(), fields);
        assert_eq!(record.location(), request.location);
        let other = GraphDiscovery::new(
            Path::new("/storage-member/main.jai"),
            GraphOptions::default(),
            &source,
        )
        .unwrap();
        assert!(other.graph().source_storage_member(id).is_none());
    }
    #[test]
    fn collision_rolls_back_storage_path_allocation() {
        let source =
            overlay("Pair::struct{value:int;} state:Pair; value::1; using state; main::(){}");
        let mut discovery = GraphDiscovery::new(
            Path::new("/storage-member/main.jai"),
            GraphOptions::default(),
            &source,
        )
        .unwrap();
        assert!(!discovery.advance().unwrap().is_complete());
        let request = discovery.pending_using_requests().pop().unwrap();
        let owner = discovery
            .graph()
            .declarations()
            .iter()
            .find(|decl| discovery.graph().symbols().name(decl.name()) == "state")
            .unwrap()
            .id();
        let name = discovery.graph().symbols().find("value").unwrap();
        let decision = FileUsingDecision {
            storage_members: vec![UsingStorageMember {
                owner,
                path: vec![name],
                destination: "value".into(),
            }],
            ..Default::default()
        };
        assert!(discovery.resolve_using(request.id, decision).is_err());
        assert!(
            discovery
                .graph()
                .source_storage_member_id(owner, &[name])
                .is_none()
        );
        assert_eq!(discovery.pending_using_requests().len(), 1);
    }
}
