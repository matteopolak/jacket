use super::*;

impl CompilerIntrinsic {
    pub(super) fn source(
        self,
        arguments: &[Value],
        effects: &mut impl CompilerEffects,
        types: &dyn TypeView,
    ) -> Result<Vec<Value>, EffectError> {
        if self == Self::SourceDebugBreak {
            if !arguments.is_empty() {
                return Err(EffectError::Failed(Error::InvalidIr(
                    "compile-time debug break requires no arguments",
                )));
            }
            return Err(EffectError::Failed(Error::RuntimeTrap));
        }
        if matches!(
            self,
            Self::SourceWriteStrings
                | Self::SourceVersionInfo { .. }
                | Self::SourceRuntimeInfo { .. }
                | Self::SourceWaitForMessage { .. }
        ) {
            return Err(EffectError::Failed(Error::InvalidIr(
                "this compiler intrinsic requires its checked VM memory adapter",
            )));
        }
        if self == Self::SourceWriteString {
            let [Value::String(bytes), Value::Bool(error)] = arguments else {
                return Err(EffectError::Failed(Error::InvalidIr(
                    "write_string requires a string and standard-error flag",
                )));
            };
            let request = CompilerRequest::WriteOutput {
                stream: if *error {
                    CompilerOutputStream::StandardError
                } else {
                    CompilerOutputStream::StandardOutput
                },
                bytes: bytes.clone(),
            };
            return match effects.request(request) {
                EffectOutcome::Ready(CompilerResponse::Unit) => Ok(vec![]),
                EffectOutcome::Ready(_) => Err(EffectError::Failed(Error::EffectResponse(
                    "write_string requires a unit response",
                ))),
                EffectOutcome::Pending(key) => Err(EffectError::Pending(Dependency::Effect(key))),
                EffectOutcome::Rejected(reason) => {
                    Err(EffectError::Failed(Error::EffectRejected(reason)))
                }
            };
        }
        if let Self::SourceGetBuildOptions {
            current_workspace,
            projection,
            result,
        } = self
        {
            return super::source_options::invoke_get_options(
                projection,
                result,
                arguments,
                current_workspace,
                effects,
                types,
            );
        }
        if matches!(
            self,
            Self::SourceAddFileAt { .. } | Self::SourceAddStringAt { .. } | Self::SourceReportAt
        ) {
            return super::source_location::invoke_location(self, arguments, effects);
        }
        if let Self::SourceSetBuildOptionsAt {
            current_workspace,
            projection,
        } = self
        {
            return super::source_options::invoke_options_at(
                projection,
                arguments,
                current_workspace,
                effects,
                types,
            );
        }
        if let Self::SourceSetBuildOptions {
            current_workspace,
            projection,
        } = self
        {
            return super::source_options::invoke_options(
                projection,
                arguments,
                current_workspace,
                effects,
                types,
            );
        }
        let fail = |reason| EffectError::Failed(Error::InvalidIr(reason));
        let text = |index| match arguments.get(index) {
            Some(Value::String(bytes)) => String::from_utf8(bytes.clone())
                .map_err(|_| fail("source compiler intrinsic requires UTF-8 text")),
            _ => Err(fail("source compiler intrinsic requires a string")),
        };
        let workspace = |index, current| match arguments.get(index) {
            Some(Value::Int(value)) if value.ty() == IntegerType::S64 && value.value() == -1 => {
                Ok(current)
            }
            Some(Value::Int(value)) if value.ty() == IntegerType::S64 && value.value() > 0 => {
                WorkspaceId::from_raw(value.bits())
                    .ok_or_else(|| fail("invalid source workspace identity"))
            }
            _ => Err(fail(
                "source workspace identity must be positive s64 or current-workspace sentinel -1",
            )),
        };
        let expected = match self {
            Self::SourceCurrentWorkspace {
                ..
            } => 0,
            Self::SourceCreateWorkspace
            | Self::SourceReport {
                ..
            }
            | Self::SourceDestroyWorkspace {
                ..
            }
            | Self::SourceGetWorkspaceName {
                ..
            } => 1,
            Self::SourceEndIntercept {
                ..
            } => 1,
            Self::SourceAddString {
                ..
            }
            | Self::SourceAddFile {
                ..
            }
            | Self::SourceSetWorkspaceStatus {
                ..
            } => 2,
            Self::SourceBeginIntercept {
                ..
            } => 2,
            _ => return Err(fail("source intrinsic adapter received internal intrinsic")),
        };
        if arguments.len() != expected {
            return Err(fail(
                "source compiler intrinsic has unsupported argument count or optional code/location suffix",
            ));
        }
        if let Self::SourceCurrentWorkspace {
            current_workspace,
        } = self
        {
            return Ok(vec![Value::Int(
                Integer::checked(IntegerType::S64, i128::from(current_workspace.get()))
                    .ok_or_else(|| fail("workspace identity exceeds source s64 ABI"))?,
            )]);
        }
        let mut fatal_message = None;
        let request = match self {
            Self::SourceBeginIntercept {
                current_workspace,
            } => {
                let Value::Enum {
                    value, ..
                } = &arguments[1]
                else {
                    return Err(fail(
                        "intercept flags require the checked source u32 flags enum",
                    ));
                };
                if value.ty() != IntegerType::U32 {
                    return Err(fail(
                        "intercept flags require the checked source u32 flags enum",
                    ));
                }
                let flags = InterceptFlags::from_bits(value.bits() as u32)
                    .ok_or_else(|| fail("intercept flags contain unknown source bits"))?;
                CompilerRequest::BeginIntercept {
                    workspace: workspace(0, current_workspace)?,
                    flags,
                }
            }
            Self::SourceEndIntercept {
                current_workspace,
            } => CompilerRequest::EndIntercept {
                workspace: workspace(0, current_workspace)?,
            },
            Self::SourceGetWorkspaceName {
                current_workspace,
            } => CompilerRequest::GetWorkspaceName {
                workspace: workspace(0, current_workspace)?,
            },
            Self::SourceDestroyWorkspace {
                current_workspace,
            } => CompilerRequest::DestroyWorkspace {
                workspace: workspace(0, current_workspace)?,
            },
            Self::SourceSetWorkspaceStatus {
                current_workspace,
            } => {
                let status = match &arguments[0] {
                    Value::Enum {
                        value, ..
                    } if value.ty() == IntegerType::U8 => match value.value() {
                        0 => WorkspaceStatus::Ok,
                        1 => WorkspaceStatus::Failed,
                        _ => return Err(fail("unknown Workspace_Status enum value")),
                    },
                    _ => return Err(fail("workspace status requires the checked source u8 enum")),
                };
                CompilerRequest::SetWorkspaceStatus {
                    workspace: workspace(1, current_workspace)?,
                    status,
                }
            }
            Self::SourceCreateWorkspace => CompilerRequest::CreateWorkspace {
                name: text(0)?,
            },
            Self::SourceAddString {
                current_workspace,
            } => CompilerRequest::AddSource {
                workspace: workspace(1, current_workspace)?,
                source: text(0)?,
            },
            Self::SourceAddFile {
                current_workspace,
            } => CompilerRequest::AddSourceFile {
                workspace: workspace(1, current_workspace)?,
                path: PathBuf::from(text(0)?),
            },
            Self::SourceReport {
                level,
            } => {
                let message = text(0)?;
                if level == MessageLevel::Error {
                    fatal_message = Some(message.clone());
                }
                CompilerRequest::Message {
                    level,
                    text: message,
                }
            }
            _ => return Err(fail("source current workspace was not handled")),
        };
        match effects.request(request) {
            EffectOutcome::Ready(CompilerResponse::WorkspaceName(name))
                if matches!(self, Self::SourceGetWorkspaceName { .. }) =>
            {
                Ok(vec![Value::String(name.into_bytes())])
            }
            EffectOutcome::Ready(CompilerResponse::Workspace(id))
                if self == Self::SourceCreateWorkspace =>
            {
                Ok(vec![Value::Int(
                    Integer::checked(IntegerType::S64, i128::from(id.get()))
                        .ok_or_else(|| fail("workspace identity exceeds source s64 ABI"))?,
                )])
            }
            EffectOutcome::Ready(CompilerResponse::Unit)
                if !matches!(
                    self,
                    Self::SourceCreateWorkspace | Self::SourceGetWorkspaceName { .. }
                ) =>
            {
                if let Some(message) = fatal_message {
                    Err(EffectError::Failed(Error::CompilerReported(message)))
                } else {
                    Ok(vec![])
                }
            }
            EffectOutcome::Ready(_) => Err(EffectError::Failed(Error::EffectResponse(
                "response kind does not match source intrinsic",
            ))),
            EffectOutcome::Pending(key) => Err(EffectError::Pending(Dependency::Effect(key))),
            EffectOutcome::Rejected(reason) => {
                Err(EffectError::Failed(Error::EffectRejected(reason)))
            }
        }
    }
}

pub(crate) fn workspace_argument(
    value: &Value,
    current: WorkspaceId,
) -> Result<WorkspaceId, EffectError> {
    match value {
        Value::Int(value) if value.ty() == IntegerType::S64 && value.value() == -1 => Ok(current),
        Value::Int(value) if value.ty() == IntegerType::S64 && value.value() > 0 => {
            WorkspaceId::from_raw(value.bits()).ok_or({
                EffectError::Failed(Error::InvalidIr("invalid source workspace identity"))
            })
        }
        _ => Err(EffectError::Failed(Error::InvalidIr(
            "source workspace identity must be positive s64 or current-workspace sentinel -1",
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Names {
        requests: Vec<CompilerRequest>,
        response: CompilerResponse,
    }
    impl CompilerEffects for Names {
        fn begin(&mut self) {
        }
        fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
            self.requests.push(request);
            EffectOutcome::Ready(self.response.clone())
        }
        fn finish(&mut self, _: bool) -> Result<(), Error> {
            Ok(())
        }
    }
    fn signed(value: i128) -> Value {
        Value::Int(Integer::checked(IntegerType::S64, value).unwrap())
    }

    #[test]
    fn interception_adapters_preserve_current_workspace_and_reject_unknown_flag_bits() {
        let mut types = jai_types::TypeRegistry::new();
        let ty = types.reserve_enum(IntegerType::U32);
        types
            .define_enum(ty, vec![Integer::checked(IntegerType::U32, 63).unwrap()])
            .unwrap();
        let current = WorkspaceId::from_raw(7).unwrap();
        let mut effects = Names {
            requests: vec![],
            response: CompilerResponse::Unit,
        };
        let begin = CompilerIntrinsic::SourceBeginIntercept {
            current_workspace: current,
        };
        let flags = |bits| Value::Enum {
            ty,
            value: Integer::checked(IntegerType::U32, bits).unwrap(),
        };
        assert!(
            begin
                .source(&[signed(-1), flags(63)], &mut effects, &types)
                .is_ok()
        );
        assert_eq!(
            effects.requests,
            vec![CompilerRequest::BeginIntercept {
                workspace: current,
                flags: InterceptFlags::SKIP_ALL
            }]
        );
        effects.requests.clear();
        assert!(
            begin
                .source(&[signed(-1), flags(64)], &mut effects, &types)
                .is_err()
        );
        assert!(effects.requests.is_empty());
        assert!(
            CompilerIntrinsic::SourceEndIntercept {
                current_workspace: current
            }
            .source(&[signed(-1)], &mut effects, &types)
            .is_ok()
        );
        assert_eq!(
            effects.requests,
            vec![CompilerRequest::EndIntercept {
                workspace: current
            }]
        );
    }

    #[test]
    fn workspace_name_uses_current_identity_and_requires_the_real_typed_response() {
        let types = jai_types::TypeRegistry::new();
        let current = WorkspaceId::from_raw(7).unwrap();
        let intrinsic = CompilerIntrinsic::SourceGetWorkspaceName {
            current_workspace: current,
        };
        let mut effects = Names {
            requests: vec![],
            response: CompilerResponse::WorkspaceName("child π".into()),
        };
        assert!(
            matches!(intrinsic.source(&[signed(-1)], &mut effects, &types), Ok(values) if values == vec![Value::String("child π".as_bytes().to_vec())])
        );
        assert_eq!(
            effects.requests,
            vec![CompilerRequest::GetWorkspaceName {
                workspace: current
            }]
        );
        effects.response = CompilerResponse::Unit;
        assert!(matches!(
            intrinsic.source(&[signed(-1)], &mut effects, &types),
            Err(EffectError::Failed(Error::EffectResponse(_)))
        ));
        effects.requests.clear();
        assert!(matches!(
            intrinsic.source(&[signed(0)], &mut effects, &types),
            Err(EffectError::Failed(Error::InvalidIr(_)))
        ));
        assert!(effects.requests.is_empty());
    }
}
