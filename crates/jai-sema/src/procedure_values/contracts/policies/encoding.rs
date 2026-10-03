//! The replay encoder supplies stable identities for checked semantic values.
use super::*;

pub(crate) trait CallablePolicyEncoder {
    type Error;
    fn token(&mut self, bytes: &[u8]);
    fn ty(&mut self, ty: TypeId) -> Result<(), Self::Error>;
    fn symbol(&mut self, symbol: Symbol);
    fn field(&mut self, field: FieldId) -> Result<(), Self::Error>;
    fn constant(&mut self, value: &jai_ir::ConstantValue) -> Result<(), Self::Error>;
    fn runtime_default(
        &mut self,
        read: &crate::runtime_defaults::RuntimeDefaultRead,
    ) -> Result<(), Self::Error>;
    fn symbol_sort_key(&self, symbol: Symbol) -> Vec<u8>;
}

impl CallablePolicyKey {
    pub(crate) fn encode<E: CallablePolicyEncoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        encoder.ty(self.ty)?;
        match &self.policy {
            ValuePolicy::Callable {
                argument_policy,
                parameters,
                variadic,
                results,
            } => {
                encoder.token(b"callable");
                encoder.token(match argument_policy {
                    CallbackArgumentPolicy::Established => b"established",
                    CallbackArgumentPolicy::Ambiguous => b"ambiguous",
                });
                count(encoder, parameters.len());
                for parameter in parameters {
                    match parameter.name {
                        Some(name) => {
                            encoder.token(b"named");
                            encoder.symbol(name);
                        }
                        None => encoder.token(b"unnamed"),
                    }
                    encoder.ty(parameter.ty)?;
                    encoder.token(match parameter.evaluation {
                        EvaluationPolicy::Evaluate => b"evaluate",
                        EvaluationPolicy::Discard => b"discard",
                    });
                    match &parameter.default {
                        None => encoder.token(b"required"),
                        Some(DefaultPolicy::PendingSource {
                            key,
                            bytes,
                        }) => {
                            encoder.token(b"pending-original-source-default");
                            encoder.token(bytes.as_bytes());
                            count(encoder, key.parameter);
                            encoder.ty(key.expected)?;
                        }
                        Some(DefaultPolicy::Constant(value)) => {
                            encoder.token(b"constant-default");
                            encoder.constant(value)?;
                        }
                        Some(DefaultPolicy::RuntimeRead(read)) => {
                            encoder.token(b"runtime-read-default");
                            encoder.runtime_default(&read.0)?;
                        }
                        Some(DefaultPolicy::CallerLocation(ty)) => {
                            encoder.token(b"caller-location-default");
                            encoder.ty(*ty)?;
                        }
                        Some(DefaultPolicy::CodeNull(ty)) => {
                            encoder.token(b"code-null-default");
                            encoder.ty(*ty)?;
                        }
                        Some(DefaultPolicy::Discarded) => encoder.token(b"discarded-default"),
                    }
                }
                match variadic {
                    VariadicPolicy::None => encoder.token(b"fixed"),
                    VariadicPolicy::Jai(position) => {
                        encoder.token(b"jai-pack");
                        count(encoder, *position);
                    }
                    VariadicPolicy::C(position) => {
                        encoder.token(b"c-pack");
                        count(encoder, *position);
                    }
                }
                count(encoder, results.len());
                for (ty, result) in results {
                    encoder.ty(*ty)?;
                    optional(encoder, result.as_ref())?;
                }
            }
            ValuePolicy::Pointer(element) => {
                encoder.token(b"pointer");
                element.encode(encoder)?;
            }
            ValuePolicy::Sequence(element) => {
                encoder.token(b"sequence");
                element.encode(encoder)?;
            }
            ValuePolicy::Record {
                bindings,
                fields,
            } => {
                encoder.token(b"record");
                bindings.encode(encoder)?;
                count(encoder, fields.len());
                for (field, policy) in fields {
                    encoder.field(*field)?;
                    optional(encoder, policy.as_ref())?;
                }
            }
            ValuePolicy::RecursiveRecord(bindings) => {
                encoder.token(b"recursive-record");
                bindings.encode(encoder)?;
            }
        }
        Ok(())
    }
}

impl RecordBindings {
    fn encode<E: CallablePolicyEncoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        let mut bindings = self
            .0
            .iter()
            .map(|(name, value)| (encoder.symbol_sort_key(*name), *name, value))
            .collect::<Vec<_>>();
        bindings.sort_by(|left, right| left.0.cmp(&right.0));
        count(encoder, bindings.len());
        for (_, name, value) in bindings {
            encoder.symbol(name);
            optional(encoder, value.as_ref())?;
        }
        Ok(())
    }
}

fn count(encoder: &mut impl CallablePolicyEncoder, count: usize) {
    encoder.token(&(count as u64).to_le_bytes());
}
fn optional<E: CallablePolicyEncoder>(
    encoder: &mut E,
    value: Option<&CallablePolicyKey>,
) -> Result<(), E::Error> {
    match value {
        Some(value) => {
            encoder.token(b"some");
            value.encode(encoder)?;
        }
        None => encoder.token(b"none"),
    }
    Ok(())
}
