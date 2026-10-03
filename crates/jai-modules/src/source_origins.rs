//! Versioned source identities for effects and captured definitions.
use super::*;
use jai_eval::Value;
use jai_types::{CallingConvention, ContextMode, FloatType, Integer, ScalarType};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceOriginError {
    UnknownFile,
    UnknownDeclaration,
    InvalidSourceSpan,
    IncompleteModule,
    UnboundValue,
    ExcessiveNesting,
}
impl fmt::Display for SourceOriginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownFile => "source environment references an unavailable file",
            Self::UnknownDeclaration => "source environment references an unavailable declaration",
            Self::InvalidSourceSpan => "source environment references an invalid declaration span",
            Self::IncompleteModule => "source environment requires a published module entry",
            Self::UnboundValue => "source environment contains an unresolved contextual value",
            Self::ExcessiveNesting => "source environment exceeds the structural nesting limit",
        })
    }
}
impl std::error::Error for SourceOriginError {}

impl ModuleGraph {
    /// Stable source bytes, without arena IDs, for the defining module's
    /// parameter environment. Source declarations include their own environments.
    pub fn module_environment_origin(
        &self,
        file: FileInstanceId,
    ) -> Result<Vec<u8>, SourceOriginError> {
        self.files
            .get(file.index())
            .ok_or(SourceOriginError::UnknownFile)?;
        let mut encoder = Encoder {
            graph: self,
            bytes: b"jai-module-environment\0\x03".to_vec(),
            modules: HashMap::new(),
            files: HashMap::new(),
        };
        encoder.target();
        encoder.file(file, 0)?;
        // Program settings may be read through another imported source module.
        // Order by source identity, never by allocation or import discovery.
        let mut program = vec![];
        for parameter in self
            .parameters
            .iter()
            .filter(|parameter| parameter.program_wide)
        {
            let mut fact = Encoder {
                graph: self,
                bytes: vec![],
                modules: HashMap::new(),
                files: HashMap::new(),
            };
            fact.data(self.symbols.name(parameter.name).as_bytes());
            fact.source_slice(parameter.location)?;
            fact.number(parameter.location.span.start as u64);
            fact.value(&parameter.value, 0)?;
            program.push(fact.bytes);
        }
        program.sort();
        program.dedup();
        encoder.number(program.len() as u64);
        for fact in program {
            encoder.data(&fact);
        }
        Ok(encoder.bytes)
    }
}

struct Encoder<'a> {
    graph: &'a ModuleGraph,
    bytes: Vec<u8>,
    modules: HashMap<ModuleId, u64>,
    files: HashMap<FileInstanceId, u64>,
}
impl Encoder<'_> {
    fn target(&mut self) {
        use jai_types::{Architecture as A, ByteOrder, OperatingSystem as O};
        let Some(target) = &self.graph.target else {
            self.tag(0);
            return;
        };
        self.tag(1);
        match &target.operating_system {
            O::MacOS => self.tag(0),
            O::Linux => self.tag(1),
            O::Windows => self.tag(2),
            O::Android => self.tag(3),
            O::IOS => self.tag(4),
            O::WebAssembly => self.tag(5),
            O::Other(name) => {
                self.tag(6);
                self.data(name.as_bytes());
            }
        }
        match &target.architecture {
            A::X86_64 => self.tag(0),
            A::Arm64 => self.tag(1),
            A::X86 => self.tag(2),
            A::Arm => self.tag(3),
            A::WebAssembly32 => self.tag(4),
            A::WebAssembly64 => self.tag(5),
            A::Other(name) => {
                self.tag(6);
                self.data(name.as_bytes());
            }
        }
        self.tag(match target.byte_order {
            ByteOrder::Little => 0,
            ByteOrder::Big => 1,
        });
        let mut key = KeyBytes::default();
        target.layout.hash(&mut key);
        self.data(&key.0);
    }
    fn tag(&mut self, tag: u8) {
        self.bytes.push(tag);
    }
    fn number(&mut self, value: u64) {
        self.bytes.extend(value.to_le_bytes());
    }
    fn data(&mut self, value: &[u8]) {
        self.number(value.len() as u64);
        self.bytes.extend(value);
    }
    fn module(&mut self, id: ModuleId, depth: usize) -> Result<(), SourceOriginError> {
        if depth > 128 {
            return Err(SourceOriginError::ExcessiveNesting);
        }
        if let Some(ordinal) = self.modules.get(&id).copied() {
            self.tag(0);
            self.number(ordinal);
            return Ok(());
        }
        let ordinal = self.modules.len() as u64;
        self.modules.insert(id, ordinal);
        self.tag(1);
        self.number(ordinal);
        let module = self
            .graph
            .module(id)
            .ok_or(SourceOriginError::IncompleteModule)?;
        let entry = module
            .discovered_entry()
            .ok_or(SourceOriginError::IncompleteModule)?;
        for file in &module.files {
            for item in self.graph.files[file.index()].syntax.items() {
                if let jai_syntax::FileItem::Parameters(header) = item {
                    for (parameter, program_wide) in header
                        .instance
                        .iter()
                        .map(|parameter| (parameter, false))
                        .chain(
                            header
                                .program
                                .iter()
                                .flatten()
                                .map(|parameter| (parameter, true)),
                        )
                    {
                        if !self.graph.parameters.iter().any(|bound| {
                            bound.module == id
                                && bound.name == parameter.name
                                && bound.location == parameter.location
                                && bound.program_wide == program_wide
                        }) {
                            return Err(SourceOriginError::IncompleteModule);
                        }
                    }
                }
            }
        }
        self.source(self.graph.files[entry.index()].source)?;
        let request = self
            .graph
            .source_requests
            .get(&id)
            .ok_or(SourceOriginError::IncompleteModule)?;
        self.data(request.path.as_os_str().as_encoded_bytes());
        match &request.arguments {
            None => self.tag(0),
            Some(arguments) => {
                self.tag(1);
                self.number(arguments.len() as u64);
                for argument in arguments {
                    match argument.name {
                        None => self.tag(0),
                        Some(name) => {
                            self.tag(1);
                            self.data(self.graph.symbols.name(name).as_bytes());
                        }
                    }
                    // Contextual member syntax is part of a valid request key;
                    // its published formal value below carries the resolved enum.
                    if let ParameterValue::ContextualMember(name) = argument.value {
                        self.tag(8);
                        self.data(self.graph.symbols.name(name).as_bytes());
                    } else {
                        self.value(&argument.value, depth + 1)?;
                    }
                }
            }
        }
        let parameters = self
            .graph
            .parameters
            .iter()
            .filter(|parameter| parameter.module == id)
            .collect::<Vec<_>>();
        self.number(parameters.len() as u64);
        for parameter in parameters {
            self.data(self.graph.symbols.name(parameter.name).as_bytes());
            self.tag(u8::from(parameter.program_wide));
            self.number(parameter.location.span.start as u64);
            self.number(parameter.location.span.end as u64);
            self.source_slice(parameter.location)?;
            self.value(&parameter.value, depth + 1)?;
        }
        Ok(())
    }
    fn location(&mut self, location: SourceSpan) -> Result<(), SourceOriginError> {
        self.source_slice(location)?;
        self.number(location.span.start as u64);
        self.number(location.span.end as u64);
        Ok(())
    }
    fn file(&mut self, id: FileInstanceId, depth: usize) -> Result<(), SourceOriginError> {
        if depth > 128 {
            return Err(SourceOriginError::ExcessiveNesting);
        }
        if let Some(ordinal) = self.files.get(&id).copied() {
            self.tag(0);
            self.number(ordinal);
            return Ok(());
        }
        let ordinal = self.files.len() as u64;
        self.files.insert(id, ordinal);
        self.tag(1);
        self.number(ordinal);
        let file = self.graph.file(id).ok_or(SourceOriginError::UnknownFile)?;
        self.source(file.source())?;
        self.module(file.module(), depth + 1)?;
        let Some(publication) = self.graph.insertion_publication(id) else {
            self.tag(0);
            return Ok(());
        };
        self.tag(1);
        self.tag(match publication.scope {
            jai_syntax::InsertScope::Captured => 0,
            jai_syntax::InsertScope::Current => 1,
        });
        self.file(publication.destination, depth + 1)?;
        self.location(publication.location)?;
        self.file(publication.code.file, depth + 1)?;
        self.file(publication.code.source_file, depth + 1)?;
        self.location(publication.code.location)?;
        self.tag(match publication.code.checks.array_bounds {
            jai_syntax::CheckPolicy::Inherited => 0,
            jai_syntax::CheckPolicy::Disabled => 1,
        });
        self.tag(match publication.code.checks.arithmetic_overflow {
            jai_syntax::CheckPolicy::Inherited => 0,
            jai_syntax::CheckPolicy::Disabled => 1,
        });
        self.tag(match publication.code.debug {
            jai_types::DebugPolicy::Emit => 0,
            jai_types::DebugPolicy::Suppress => 1,
        });
        self.number(publication.code.origins.len() as u64);
        for location in &publication.code.origins {
            self.location(*location)?;
        }
        let mut bindings = publication.code.bindings.iter().collect::<Vec<_>>();
        bindings.sort_by_key(|(name, _)| self.graph.symbols.name(*name));
        self.number(bindings.len() as u64);
        for (name, binding) in bindings {
            self.data(self.graph.symbols.name(*name).as_bytes());
            self.binding(*binding, depth + 1)?;
        }
        let mut values = publication.code.values.iter().collect::<Vec<_>>();
        values.sort_by_key(|(name, _)| self.graph.symbols.name(*name));
        self.number(values.len() as u64);
        for (name, value) in values {
            self.data(self.graph.symbols.name(*name).as_bytes());
            self.capture(value, depth + 1)?;
        }
        Ok(())
    }
    fn capture(
        &mut self,
        value: &SourceCaptureValue,
        depth: usize,
    ) -> Result<(), SourceOriginError> {
        match value {
            SourceCaptureValue::Scalar(value) => {
                self.value(&ParameterValue::Scalar(value.clone()), depth)
            }
            SourceCaptureValue::Type(value) => {
                self.value(&ParameterValue::Type(value.clone()), depth)
            }
            SourceCaptureValue::Enumeration(value) => {
                self.value(&ParameterValue::Enumeration(*value), depth)
            }
            SourceCaptureValue::String(value) => {
                self.tag(5);
                self.data(value);
                Ok(())
            }
        }
    }
    fn binding(&mut self, binding: Binding, depth: usize) -> Result<(), SourceOriginError> {
        match binding {
            Binding::Declaration(id) => {
                self.tag(0);
                self.declaration(id, depth)
            }
            Binding::OverloadSet(id) => {
                self.tag(1);
                let set = self
                    .graph
                    .overload_set(id)
                    .ok_or(SourceOriginError::UnknownDeclaration)?;
                self.number(set.declarations().len() as u64);
                for id in set.declarations() {
                    self.declaration(*id, depth + 1)?;
                }
                Ok(())
            }
            Binding::Module(id) => {
                self.tag(2);
                self.module(id, depth)
            }
            Binding::Parameter(id) => {
                self.tag(3);
                let parameter = self
                    .graph
                    .parameter(id)
                    .ok_or(SourceOriginError::UnboundValue)?;
                self.data(self.graph.symbols.name(parameter.name).as_bytes());
                self.location(parameter.location)?;
                self.tag(u8::from(parameter.program_wide));
                self.module(parameter.module, depth + 1)?;
                self.value(&parameter.value, depth + 1)
            }
            Binding::StorageMember(id) => {
                self.tag(4);
                let member = self
                    .graph
                    .source_storage_member(id)
                    .ok_or(SourceOriginError::UnknownDeclaration)?;
                self.declaration(member.owner(), depth + 1)?;
                self.number(member.path().len() as u64);
                for name in member.path() {
                    self.data(self.graph.symbols.name(*name).as_bytes());
                }
                Ok(())
            }
            Binding::SourceMember {
                declaration,
                member,
            } => {
                self.tag(5);
                self.declaration(declaration, depth + 1)?;
                self.data(self.graph.symbols.name(member).as_bytes());
                Ok(())
            }
        }
    }
    fn source(&mut self, id: SourceId) -> Result<(), SourceOriginError> {
        let source = self
            .graph
            .sources
            .get(id)
            .ok_or(SourceOriginError::UnknownFile)?;
        self.data(source.path().as_os_str().as_encoded_bytes());
        Ok(())
    }
    fn source_slice(&mut self, location: SourceSpan) -> Result<(), SourceOriginError> {
        self.source(location.source)?;
        let source = self
            .graph
            .sources
            .get(location.source)
            .ok_or(SourceOriginError::UnknownFile)?;
        let relevant = source
            .text()
            .get(location.span.start..location.span.end)
            .ok_or(SourceOriginError::InvalidSourceSpan)?;
        self.data(relevant.as_bytes());
        Ok(())
    }
    fn declaration(&mut self, id: DeclarationId, depth: usize) -> Result<(), SourceOriginError> {
        let declaration = self
            .graph
            .declaration(id)
            .ok_or(SourceOriginError::UnknownDeclaration)?;
        self.source_slice(declaration.location())?;
        self.number(declaration.location().span.start as u64);
        self.number(declaration.location().span.end as u64);
        self.data(self.graph.symbols.name(declaration.name()).as_bytes());
        self.file(declaration.file, depth + 1)
    }
    fn integer(&mut self, value: Integer) {
        self.tag(u8::from(value.ty().signed()));
        self.number(u64::from(value.ty().bits()));
        self.number(value.bits());
    }
    fn float(&mut self, value: jai_types::FloatValue) {
        self.tag(match value.ty() {
            FloatType::F32 => 0,
            FloatType::F64 => 1,
        });
        self.number(value.bits());
    }
    fn value(&mut self, value: &ParameterValue, depth: usize) -> Result<(), SourceOriginError> {
        match value {
            ParameterValue::Scalar(Value::Literal(value)) => {
                self.tag(0);
                self.bytes.extend(value.to_le_bytes());
            }
            ParameterValue::Scalar(Value::Int(value)) => {
                self.tag(1);
                self.integer(*value);
            }
            ParameterValue::Scalar(Value::Bool(value)) => {
                self.tag(2);
                self.tag(u8::from(*value));
            }
            ParameterValue::Scalar(Value::Float(value)) => {
                self.tag(3);
                self.float(*value);
            }
            ParameterValue::Scalar(Value::WeakFloat(value)) => {
                self.tag(4);
                let mut key = KeyBytes::default();
                value.request_key().hash(&mut key);
                self.data(&key.0);
            }
            ParameterValue::String(value) => {
                self.tag(5);
                self.data(value.as_bytes());
            }
            ParameterValue::Enumeration(value) => {
                self.tag(6);
                self.declaration(value.declaration, depth)?;
                self.integer(value.value);
            }
            ParameterValue::Type(value) => {
                self.tag(7);
                self.ty(value, depth)?;
            }
            ParameterValue::ContextualMember(_) => return Err(SourceOriginError::UnboundValue),
        }
        Ok(())
    }
    fn argument(
        &mut self,
        value: &ModuleBoundArgument,
        depth: usize,
    ) -> Result<(), SourceOriginError> {
        match value {
            ModuleBoundArgument::Type(value) => {
                self.tag(0);
                self.ty(value, depth)?;
            }
            ModuleBoundArgument::Integer(value) => {
                self.tag(1);
                self.integer(*value);
            }
            ModuleBoundArgument::Boolean(value) => {
                self.tag(2);
                self.tag(u8::from(*value));
            }
            ModuleBoundArgument::Float(value) => {
                self.tag(3);
                self.float(*value);
            }
            ModuleBoundArgument::Enumeration(value) => {
                self.tag(4);
                self.declaration(value.declaration, depth)?;
                self.integer(value.value);
            }
            ModuleBoundArgument::String(value) => {
                self.tag(5);
                self.data(value);
            }
        }
        Ok(())
    }
    fn ty(&mut self, ty: &ModuleType, depth: usize) -> Result<(), SourceOriginError> {
        if depth > 128 {
            return Err(SourceOriginError::ExcessiveNesting);
        }
        match ty {
            ModuleType::Builtin(ty) => {
                self.tag(0);
                match ty {
                    ModuleBuiltin::Scalar(ScalarType::Int(ty)) => {
                        self.tag(0);
                        self.tag(u8::from(ty.signed()));
                        self.number(u64::from(ty.bits()));
                    }
                    ModuleBuiltin::Scalar(ScalarType::Bool) => self.tag(1),
                    ModuleBuiltin::Float(FloatType::F32) => self.tag(2),
                    ModuleBuiltin::Float(FloatType::F64) => self.tag(3),
                    ModuleBuiltin::String => self.tag(4),
                    ModuleBuiltin::Void => self.tag(5),
                    ModuleBuiltin::Type => self.tag(6),
                    ModuleBuiltin::Context => self.tag(7),
                    ModuleBuiltin::Any => self.tag(8),
                }
            }
            ModuleType::Declaration(id) => {
                self.tag(1);
                self.declaration(*id, depth)?;
            }
            ModuleType::Application {
                template,
                arguments,
            } => {
                self.tag(2);
                self.declaration(*template, depth)?;
                self.number(arguments.len() as u64);
                for argument in arguments {
                    self.argument(argument, depth + 1)?;
                }
            }
            ModuleType::Pointer(inner) => {
                self.tag(3);
                self.ty(inner, depth + 1)?;
            }
            ModuleType::Slice(inner) => {
                self.tag(4);
                self.ty(inner, depth + 1)?;
            }
            ModuleType::DynamicArray(inner) => {
                self.tag(5);
                self.ty(inner, depth + 1)?;
            }
            ModuleType::FixedArray { element, count } => {
                self.tag(6);
                self.number(*count);
                self.ty(element, depth + 1)?;
            }
            ModuleType::Procedure(procedure) => {
                self.tag(7);
                self.tag(match procedure.convention {
                    CallingConvention::Jai => 0,
                    CallingConvention::C => 1,
                    CallingConvention::Stdcall => 2,
                    CallingConvention::CppMethod => 3,
                });
                self.tag(match procedure.context {
                    ContextMode::Implicit => 0,
                    ContextMode::None => 1,
                });
                match procedure.variadic {
                    ModuleVariadic::None => self.tag(0),
                    ModuleVariadic::C { fixed_parameters } => {
                        self.tag(1);
                        self.number(fixed_parameters as u64);
                    }
                    ModuleVariadic::Jai { parameter } => {
                        self.tag(2);
                        self.number(parameter as u64);
                    }
                }
                self.number(procedure.parameters.len() as u64);
                for ty in &procedure.parameters {
                    self.ty(ty, depth + 1)?;
                }
                self.number(procedure.results.len() as u64);
                for ty in &procedure.results {
                    self.ty(ty, depth + 1)?;
                }
            }
        }
        Ok(())
    }
}

/// Retain every framed canonical key write; never reduce exact weak decimals to
/// a rounded float or to a collision-prone digest.
#[derive(Default)]
struct KeyBytes(Vec<u8>);
impl Hasher for KeyBytes {
    fn finish(&self) -> u64 {
        0
    }
    fn write(&mut self, bytes: &[u8]) {
        self.0.extend((bytes.len() as u64).to_le_bytes());
        self.0.extend(bytes);
    }
    fn write_usize(&mut self, value: usize) {
        self.write(&(value as u64).to_le_bytes());
    }
    fn write_isize(&mut self, value: isize) {
        self.write(&(value as i64).to_le_bytes());
    }
    fn write_u8(&mut self, value: u8) {
        self.write(&value.to_le_bytes());
    }
    fn write_u16(&mut self, value: u16) {
        self.write(&value.to_le_bytes());
    }
    fn write_u32(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }
    fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }
    fn write_u128(&mut self, value: u128) {
        self.write(&value.to_le_bytes());
    }
    fn write_i8(&mut self, value: i8) {
        self.write(&value.to_le_bytes());
    }
    fn write_i16(&mut self, value: i16) {
        self.write(&value.to_le_bytes());
    }
    fn write_i32(&mut self, value: i32) {
        self.write(&value.to_le_bytes());
    }
    fn write_i64(&mut self, value: i64) {
        self.write(&value.to_le_bytes());
    }
    fn write_i128(&mut self, value: i128) {
        self.write(&value.to_le_bytes());
    }
}
