//! Drive real subscribed source jobs under their exact parent transaction.
use super::*;

impl WorkspaceScheduler {
    /// Advance subscribed child work in this source job's private preview.
    /// Native outputs remain pending until their actual backend job completes.
    pub fn service_suspended(
        &mut self,
        session: &mut CompilerSession,
        origin: &SourceOrigin,
    ) -> Result<bool, SchedulerError> {
        let mut replay = std::mem::take(&mut self.replay);
        let result = self.advance_suspended(session, &mut replay, origin);
        self.replay = replay;
        result
    }
    pub(crate) fn advance_suspended(
        &mut self,
        session: &mut CompilerSession,
        replay: &mut EffectReplayCache,
        origin: &SourceOrigin,
    ) -> Result<bool, SchedulerError> {
        if self.session.is_some_and(|id| id != session.root()) {
            return Err(SchedulerError::DifferentSession);
        }
        self.session = Some(session.root());
        self.check_physical()?;
        let id = replay.suspended_job(origin).ok_or_else(|| {
            jai_vm::Error::EffectRejected(
                "no retained parent compiler job owns this source wait".into(),
            )
        })?;
        let preview = replay.preview_suspended(origin, session)?;
        let receivers = replay.suspended_workspaces(origin);
        let retained = self.children.borrow_mut().remove(&id);
        let mut frame = match retained {
            Some(mut frame) => {
                let unchanged = frame.base_revision == preview.committed_revision()
                    && frame.state.selection.as_ref().is_some_and(|selection| {
                        selection.parent == *origin && selection.receivers == receivers
                    });
                if !unchanged {
                    self.cancel_frame(&mut frame)?;
                    return Err(SchedulerError::ChangedPendingSession);
                }
                frame
            }
            None => BuildFrame {
                base_revision: preview.committed_revision(),
                compiler: preview.clone(),
                replay: replay.fork_suspended_replay(origin)?,
                state: BuildState::new(
                    &preview,
                    Some(PreviewSelection {
                        parent: origin.clone(),
                        receivers: receivers.clone(),
                    }),
                ),
            },
        };
        let result = self.drive_build(&mut frame.compiler, &mut frame.replay, &mut frame.state);
        let build = match result {
            Ok(SchedulerReadiness::Pending(_)) => {
                self.children.borrow_mut().insert(id, frame);
                return Ok(false);
            }
            Ok(SchedulerReadiness::Complete(build)) => build,
            Err(error) => {
                self.cancel_frame(&mut frame)?;
                return Err(error);
            }
        };
        let failed = !frame.state.failed.is_empty();
        if failed {
            // Earlier source rounds belong to this same child build. A later
            // hard failure discards their inputs, outputs and replay together.
            let failures: Vec<_> = frame
                .state
                .failed
                .values()
                .map(|(workspace, _)| {
                    (
                        *workspace,
                        frame
                            .compiler
                            .source_failure(*workspace)
                            .expect("failed source records its actual compiler message")
                            .clone(),
                    )
                })
                .collect();
            frame.compiler = preview;
            frame.replay = replay.fork_suspended_replay(origin)?;
            for (workspace, message) in failures {
                frame
                    .compiler
                    .record_source_failure(workspace, &Error::CompilerReport(message))?;
            }
        }
        let mut events = vec![];
        for (workspace, source_parsed) in frame.state.failed.values() {
            if *source_parsed {
                events.push(CompilerEvent::Phase {
                    workspace: *workspace,
                    phase: CompilerPhase::SourceParsed,
                });
            }
            events.push(CompilerEvent::Complete {
                workspace: *workspace,
                error: jai_vm::CompilerCompletion::CompilationFailed,
            });
        }
        for workspace in build.workspaces.into_iter().filter(|_| !failed) {
            let WorkspaceOutput::Checked {
                ..
            } = workspace.output
            else {
                continue;
            };
            let id = workspace.id;
            if frame.compiler.is_destroyed(id) {
                events.push(CompilerEvent::Complete {
                    workspace: id,
                    error: jai_vm::CompilerCompletion::CompilerShutdown,
                });
                continue;
            }
            events.push(CompilerEvent::Phase {
                workspace: id,
                phase: CompilerPhase::SourceParsed,
            });
            events.push(CompilerEvent::Phase {
                workspace: id,
                phase: CompilerPhase::Typechecked {
                    pending_count: 0,
                },
            });
            let completed = frame
                .compiler
                .workspace(id)
                .expect("checked child still exists");
            if completed.status() == jai_vm::WorkspaceStatus::Failed {
                events.push(CompilerEvent::Complete {
                    workspace: id,
                    error: jai_vm::CompilerCompletion::CompilationFailed,
                });
            } else if completed.settings().output_kind == jai_types::BuildOutputKind::None {
                events.push(CompilerEvent::Complete {
                    workspace: id,
                    error: jai_vm::CompilerCompletion::None,
                });
            }
        }
        for id in receivers {
            if frame.compiler.is_destroyed(id) {
                events.push(CompilerEvent::Complete {
                    workspace: id,
                    error: jai_vm::CompilerCompletion::CompilerShutdown,
                });
            }
        }
        replay.prepare_suspended_preview_with_replay(origin, frame.compiler, frame.replay)?;
        let mut ready = false;
        for event in events {
            ready |= replay.publish_suspended_event(origin, event)?;
        }
        Ok(ready)
    }
}
