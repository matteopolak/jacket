use super::*;

pub(super) fn location(value: &Value) -> Result<SourceLocation, EffectError> {
    let fail = |reason| EffectError::Failed(Error::InvalidIr(reason));
    let Value::Record {
        fields, ..
    } = value
    else {
        return Err(fail("source location must be a checked record"));
    };
    if fields.len() != 3 {
        return Err(fail(
            "source location requires filename, line and character fields",
        ));
    }
    let Value::String(path) = &fields[0] else {
        return Err(fail("source location filename must be string"));
    };
    let path = String::from_utf8(path.clone())
        .map_err(|_| fail("source location filename must be UTF-8"))?;
    let number = |index| match &fields[index] {
        Value::Int(value) if value.ty() == IntegerType::S64 && value.value() >= 0 => {
            Ok(value.bits())
        }
        _ => Err(fail(
            "source location line/character must be nonnegative s64",
        )),
    };
    Ok(SourceLocation {
        path: PathBuf::from(path),
        line: number(1)?,
        column: number(2)?,
    })
}
pub(super) fn invoke_location(
    intrinsic: CompilerIntrinsic,
    arguments: &[Value],
    effects: &mut impl CompilerEffects,
) -> Result<Vec<Value>, EffectError> {
    let fail = |reason| EffectError::Failed(Error::InvalidIr(reason));
    if arguments.len() != 3 {
        return Err(fail("source location intrinsic requires three arguments"));
    }
    let Value::String(text) = &arguments[0] else {
        return Err(fail("source location intrinsic requires string text"));
    };
    let text = String::from_utf8(text.clone())
        .map_err(|_| fail("source location intrinsic requires UTF-8 text"))?;
    let mut fatal = None;
    let request = match intrinsic {
        CompilerIntrinsic::SourceAddStringAt {
            current_workspace,
        } => {
            let workspace = super::source::workspace_argument(&arguments[1], current_workspace)?;
            CompilerRequest::AddSourceAt {
                workspace,
                source: text,
                location: location(&arguments[2])?,
            }
        }
        CompilerIntrinsic::SourceAddFileAt {
            current_workspace,
        } => {
            let workspace = super::source::workspace_argument(&arguments[1], current_workspace)?;
            CompilerRequest::AddSourceFileAt {
                workspace,
                path: PathBuf::from(text),
                location: location(&arguments[2])?,
            }
        }
        CompilerIntrinsic::SourceReportAt => {
            let mode = match &arguments[2] {
                Value::Enum {
                    value, ..
                } if value.ty() == IntegerType::U8 => value.value(),
                _ => {
                    return Err(fail(
                        "source report mode requires the checked u8 Report enum",
                    ));
                }
            };
            let (level, continuation) = match mode {
                0 => (MessageLevel::Error, ReportContinuation::Stop),
                1 => (MessageLevel::Error, ReportContinuation::Continue),
                2 => (MessageLevel::Warning, ReportContinuation::Continue),
                3 => (MessageLevel::Info, ReportContinuation::Continue),
                _ => return Err(fail("unknown source Report enum value")),
            };
            let location = location(&arguments[1])?;
            if level == MessageLevel::Error && continuation == ReportContinuation::Stop {
                fatal = Some((location.clone(), text.clone()));
            }
            CompilerRequest::Report {
                level,
                continuation,
                location,
                text,
            }
        }
        _ => return Err(fail("invalid source location intrinsic")),
    };
    match effects.request(request) {
        EffectOutcome::Ready(CompilerResponse::Unit) => {
            if let Some((location, message)) = fatal {
                Err(EffectError::Failed(Error::CompilerDiagnostic {
                    location,
                    message,
                }))
            } else {
                Ok(vec![])
            }
        }
        EffectOutcome::Ready(_) => Err(EffectError::Failed(Error::EffectResponse(
            "source location effect requires Unit response",
        ))),
        EffectOutcome::Pending(key) => Err(EffectError::Pending(Dependency::Effect(key))),
        EffectOutcome::Rejected(reason) => Err(EffectError::Failed(Error::EffectRejected(reason))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{Integer, RecordKind, TypeRegistry};
    #[derive(Default)]
    struct Effects(Vec<CompilerRequest>);
    impl CompilerEffects for Effects {
        fn begin(&mut self) {
        }
        fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
            self.0.push(request);
            EffectOutcome::Ready(CompilerResponse::Unit)
        }
        fn finish(&mut self, _: bool) -> Result<(), Error> {
            Ok(())
        }
    }
    #[test]
    fn root_source_adapter_retains_order_signed_sentinel_and_location() {
        let mut types = TypeRegistry::new();
        let ty = types.reserve_record(RecordKind::Struct);
        let s64 = types.scalar(jai_types::ScalarType::Int(IntegerType::S64));
        types
            .define_record(ty, vec![types.string(), s64, s64])
            .unwrap();
        let integer = |number| Value::Int(Integer::checked(IntegerType::S64, number).unwrap());
        let workspace = WorkspaceId::from_raw(17).unwrap();
        let location = Value::Record {
            ty,
            fields: vec![
                Value::String(b"recipe/build.jai".to_vec()),
                integer(8),
                integer(13),
            ],
        };
        let mut effects = Effects::default();
        invoke_location(
            CompilerIntrinsic::SourceAddStringAt {
                current_workspace: workspace,
            },
            &[
                Value::String(b"answer :: 42;".to_vec()),
                integer(-1),
                location,
            ],
            &mut effects,
        )
        .unwrap_or_else(|_| panic!("root source with checked source location"));
        assert_eq!(
            effects.0,
            vec![CompilerRequest::AddSourceAt {
                workspace,
                source: "answer :: 42;".into(),
                location: SourceLocation {
                    path: "recipe/build.jai".into(),
                    line: 8,
                    column: 13
                },
            }]
        );
    }
}
