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
    CapacityOverflow,
    Allocation,
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
            Self::CapacityOverflow => "source environment storage capacity overflow",
            Self::Allocation => "source environment storage allocation failed",
        })
    }
}
impl std::error::Error for SourceOriginError {
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceOriginAdmissionError<E> {
    Origin(SourceOriginError),
    Admission(E),
}
impl<E> From<SourceOriginError> for SourceOriginAdmissionError<E> {
    fn from(error: SourceOriginError) -> Self {
        Self::Origin(error)
    }
}
impl ModuleGraph {
    /// Existing convenience API and metered callers use the same exact encoder.
    pub fn module_environment_origin(
        &self,
        file: FileInstanceId,
    ) -> Result<Vec<u8>, SourceOriginError> {
        match self.module_environment_origin_with_work(file, &mut |_, _| {
            Ok::<_, std::convert::Infallible>(())
        }) {
            Ok(bytes) => Ok(bytes),
            Err(SourceOriginAdmissionError::Origin(error)) => Err(error),
            Err(SourceOriginAdmissionError::Admission(never)) => match never {},
        }
    }
    /// Admit every borrowed inspection and real scratch/output growth before it
    /// occurs. Arena IDs are local traversal references, never output authority.
    pub fn module_environment_origin_with_work<E>(
        &self,
        file: FileInstanceId,
        admit: &mut impl FnMut(usize, usize) -> Result<(), E>,
    ) -> Result<Vec<u8>, SourceOriginAdmissionError<E>> {
        debit(admit, 1, 0)?;
        self.files
            .get(file.index())
            .ok_or(SourceOriginError::UnknownFile)?;
        let mut encoder = Encoder::new(self, admit)?;
        encoder.append(b"jai-module-environment\0\x04")?;
        encoder.target()?;
        encoder.file(file, 0)?;
        debit(encoder.admit, 1, std::mem::size_of::<Vec<Vec<u8>>>())?;
        let mut program = Vec::new();
        for parameter in &self.parameters {
            encoder.step()?;
            if !parameter.program_wide {
                continue;
            }
            let mut fact = Encoder::new(self, encoder.admit)?;
            fact.data(self.symbols.name(parameter.name).as_bytes())?;
            fact.source_slice(parameter.location)?;
            fact.number(parameter.location.span.start as u64)?;
            fact.value(&parameter.value, 0)?;
            let bytes = fact.bytes;
            push(&mut program, bytes, encoder.admit)?;
        }
        // Insertion sort makes every comparison's actual byte work visible.
        for index in 1..program.len() {
            let mut next = index;
            while next > 0 {
                encoder.compare_bytes(&program[next - 1], &program[next])?;
                if program[next - 1] <= program[next] {
                    break;
                }
                program.swap(next - 1, next);
                next -= 1;
            }
        }
        let mut index = 1;
        while index < program.len() {
            encoder.compare_bytes(&program[index - 1], &program[index])?;
            if program[index - 1] == program[index] {
                // Vec::remove shifts genuine inline slots; admit before moving.
                debit(encoder.admit, program.len() - index, 0)?;
                program.remove(index);
            } else {
                index += 1;
            }
        }
        encoder.number(program.len() as u64)?;
        for fact in program {
            encoder.data(&fact)?;
        }
        Ok(encoder.bytes)
    }
}

struct Encoder<'a, 'm, E> {
    graph: &'a ModuleGraph,
    bytes: Vec<u8>,
    modules: Vec<(ModuleId, u64)>,
    files: Vec<(FileInstanceId, u64)>,
    admit: &'m mut dyn FnMut(usize, usize) -> Result<(), E>,
}
impl<'a, 'm, E> Encoder<'a, 'm, E> {
    fn new(
        graph: &'a ModuleGraph,
        admit: &'m mut dyn FnMut(usize, usize) -> Result<(), E>,
    ) -> Result<Self, SourceOriginAdmissionError<E>> {
        debit(admit, 1, std::mem::size_of::<Self>())?;
        Ok(Self {
            graph,
            bytes: Vec::new(),
            modules: Vec::new(),
            files: Vec::new(),
            admit,
        })
    }
    fn target(&mut self) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        use jai_types::{Architecture as A, ByteOrder, OperatingSystem as O};
        let Some(target) = &self.graph.target else {
            self.tag(0)?;
            return Ok(());
        };
        self.tag(1)?;
        match &target.operating_system {
            O::MacOS => self.tag(0)?,
            O::Linux => self.tag(1)?,
            O::Windows => self.tag(2)?,
            O::Android => self.tag(3)?,
            O::IOS => self.tag(4)?,
            O::WebAssembly => self.tag(5)?,
            O::Other(name) => {
                self.tag(6)?;
                self.data(name.as_bytes())?;
            }
        }
        match &target.architecture {
            A::X86_64 => self.tag(0)?,
            A::Arm64 => self.tag(1)?,
            A::X86 => self.tag(2)?,
            A::Arm => self.tag(3)?,
            A::WebAssembly32 => self.tag(4)?,
            A::WebAssembly64 => self.tag(5)?,
            A::Other(name) => {
                self.tag(6)?;
                self.data(name.as_bytes())?;
            }
        }
        self.tag(match target.byte_order {
            ByteOrder::Little => 0,
            ByteOrder::Big => 1,
        })?;
        // LayoutPolicy's fixed scalar fields have a bounded derived Hash walk.
        // Charge that complete borrowed walk before entering its infallible API.
        debit(
            self.admit,
            std::mem::size_of_val(&target.layout)
                .saturating_mul(4)
                .saturating_add(16),
            std::mem::size_of::<KeyBytes<'_, E>>(),
        )?;
        let mut key = KeyBytes {
            bytes: Vec::new(),
            admit: self.admit,
            error: None,
        };
        target.layout.hash(&mut key);
        let bytes = key.complete()?;
        self.data(&bytes)?;
        Ok(())
    }
    fn tag(&mut self, tag: u8) -> Result<(), SourceOriginAdmissionError<E>> {
        self.append(&[tag])
    }
    fn number(&mut self, value: u64) -> Result<(), SourceOriginAdmissionError<E>> {
        self.append(&value.to_le_bytes())
    }
    fn data(&mut self, value: &[u8]) -> Result<(), SourceOriginAdmissionError<E>> {
        self.number(value.len() as u64)?;
        self.append(value)
    }
    fn append(&mut self, value: &[u8]) -> Result<(), SourceOriginAdmissionError<E>> {
        debit(self.admit, value.len().saturating_add(1), 0)?;
        reserve(&mut self.bytes, value.len(), self.admit)?;
        self.bytes.extend_from_slice(value);
        Ok(())
    }
    fn step(&mut self) -> Result<(), SourceOriginAdmissionError<E>> {
        debit(self.admit, 1, 0)
    }
    fn compare_bytes(&mut self, a: &[u8], b: &[u8]) -> Result<(), SourceOriginAdmissionError<E>> {
        debit(
            self.admit,
            a.len().saturating_add(b.len()).saturating_add(1),
            0,
        )
    }
    fn module_ordinal(
        &mut self,
        id: ModuleId,
    ) -> Result<Option<u64>, SourceOriginAdmissionError<E>> {
        for (old, ordinal) in &self.modules {
            debit(self.admit, 1, 0)?;
            if *old == id {
                return Ok(Some(*ordinal));
            }
        }
        Ok(None)
    }
    fn file_ordinal(
        &mut self,
        id: FileInstanceId,
    ) -> Result<Option<u64>, SourceOriginAdmissionError<E>> {
        for (old, ordinal) in &self.files {
            debit(self.admit, 1, 0)?;
            if *old == id {
                return Ok(Some(*ordinal));
            }
        }
        Ok(None)
    }
    fn references<'b, T>(
        &mut self,
        values: &'b [T],
    ) -> Result<Vec<&'b T>, SourceOriginAdmissionError<E>> {
        debit(self.admit, 1, std::mem::size_of::<Vec<&T>>())?;
        let mut result = Vec::new();
        reserve(&mut result, values.len(), self.admit)?;
        for value in values {
            self.step()?;
            result.push(value);
        }
        Ok(result)
    }
    fn sort_symbols<T>(
        &mut self,
        values: &mut [&(Symbol, T)],
    ) -> Result<(), SourceOriginAdmissionError<E>> {
        for index in 1..values.len() {
            let mut next = index;
            while next > 0 {
                let a = self.graph.symbols.name(values[next - 1].0);
                let b = self.graph.symbols.name(values[next].0);
                self.compare_bytes(a.as_bytes(), b.as_bytes())?;
                if a <= b {
                    break;
                }
                values.swap(next - 1, next);
                next -= 1;
            }
        }
        Ok(())
    }
    fn module(&mut self, id: ModuleId, depth: usize) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        if depth > 128 {
            return Err(SourceOriginError::ExcessiveNesting.into());
        }
        if let Some(ordinal) = self.module_ordinal(id)? {
            self.tag(0)?;
            self.number(ordinal)?;
            return Ok(());
        }
        let ordinal = self.modules.len() as u64;
        push(&mut self.modules, (id, ordinal), self.admit)?;
        self.tag(1)?;
        self.number(ordinal)?;
        let module = self
            .graph
            .module(id)
            .ok_or(SourceOriginError::IncompleteModule)?;
        let entry = module
            .discovered_entry()
            .ok_or(SourceOriginError::IncompleteModule)?;
        for file in &module.files {
            self.step()?;
            for item in self.graph.files[file.index()].syntax.items() {
                self.step()?;
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
                        self.step()?;
                        let mut found = false;
                        for bound in &self.graph.parameters {
                            self.step()?;
                            if bound.module == id
                                && bound.name == parameter.name
                                && bound.location == parameter.location
                                && bound.program_wide == program_wide
                            {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            return Err(SourceOriginError::IncompleteModule.into());
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
        self.data(request.path.as_os_str().as_encoded_bytes())?;
        match &request.arguments {
            None => self.tag(0)?,
            Some(arguments) => {
                self.tag(1)?;
                self.number(arguments.len() as u64)?;
                for argument in arguments {
                    self.step()?;
                    match argument.name {
                        None => self.tag(0)?,
                        Some(name) => {
                            self.tag(1)?;
                            self.data(self.graph.symbols.name(name).as_bytes())?;
                        }
                    }
                    // Contextual member syntax is part of a valid request key;
                    // its published formal value below carries the resolved enum.
                    if let ParameterValue::ContextualMember(name) = argument.value {
                        self.tag(8)?;
                        self.data(self.graph.symbols.name(name).as_bytes())?;
                    } else {
                        self.value(&argument.value, depth + 1)?;
                    }
                }
            }
        }
        let mut count = 0usize;
        for parameter in &self.graph.parameters {
            self.step()?;
            if parameter.module == id {
                count += 1;
            }
        }
        self.number(count as u64)?;
        for parameter in &self.graph.parameters {
            self.step()?;
            if parameter.module != id {
                continue;
            }
            self.data(self.graph.symbols.name(parameter.name).as_bytes())?;
            self.tag(u8::from(parameter.program_wide))?;
            self.number(parameter.location.span.start as u64)?;
            self.number(parameter.location.span.end as u64)?;
            self.source_slice(parameter.location)?;
            self.value(&parameter.value, depth + 1)?;
        }
        Ok(())
    }
    fn location(&mut self, location: SourceSpan) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        self.source_slice(location)?;
        self.number(location.span.start as u64)?;
        self.number(location.span.end as u64)?;
        Ok(())
    }
    fn file(
        &mut self,
        id: FileInstanceId,
        depth: usize,
    ) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        if depth > 128 {
            return Err(SourceOriginError::ExcessiveNesting.into());
        }
        if let Some(ordinal) = self.file_ordinal(id)? {
            self.tag(0)?;
            self.number(ordinal)?;
            return Ok(());
        }
        let ordinal = self.files.len() as u64;
        push(&mut self.files, (id, ordinal), self.admit)?;
        self.tag(1)?;
        self.number(ordinal)?;
        let file = self.graph.file(id).ok_or(SourceOriginError::UnknownFile)?;
        self.source(file.source())?;
        self.module(file.module(), depth + 1)?;
        let Some(publication) = self.graph.insertion_publication(id) else {
            self.tag(0)?;
            return Ok(());
        };
        self.tag(1)?;
        self.tag(match publication.scope {
            jai_syntax::InsertScope::Captured => 0,
            jai_syntax::InsertScope::Current => 1,
        })?;
        self.file(publication.destination, depth + 1)?;
        self.location(publication.location)?;
        self.file(publication.code.file, depth + 1)?;
        self.file(publication.code.source_file, depth + 1)?;
        self.location(publication.code.location)?;
        self.tag(match publication.code.checks.array_bounds {
            jai_syntax::CheckPolicy::Inherited => 0,
            jai_syntax::CheckPolicy::Disabled => 1,
        })?;
        self.tag(match publication.code.checks.arithmetic_overflow {
            jai_syntax::CheckPolicy::Inherited => 0,
            jai_syntax::CheckPolicy::Disabled => 1,
        })?;
        self.tag(match publication.code.debug {
            jai_types::DebugPolicy::Emit => 0,
            jai_types::DebugPolicy::Suppress => 1,
        })?;
        self.number(publication.code.origins.len() as u64)?;
        for location in &publication.code.origins {
            self.step()?;
            self.location(*location)?;
        }
        let mut bindings = self.references(&publication.code.bindings)?;
        self.sort_symbols(&mut bindings)?;
        self.number(bindings.len() as u64)?;
        for (name, binding) in bindings {
            self.step()?;
            self.data(self.graph.symbols.name(*name).as_bytes())?;
            self.binding(*binding, depth + 1)?;
        }
        let mut values = self.references(&publication.code.values)?;
        self.sort_symbols(&mut values)?;
        self.number(values.len() as u64)?;
        for (name, value) in values {
            self.step()?;
            self.data(self.graph.symbols.name(*name).as_bytes())?;
            self.capture(value, depth + 1)?;
        }
        Ok(())
    }
    fn capture(
        &mut self,
        value: &SourceCaptureValue,
        depth: usize,
    ) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        match value {
            SourceCaptureValue::Scalar(value) => self.scalar(value),
            SourceCaptureValue::Type(value) => {
                self.tag(7)?;
                self.ty(value, depth)
            }
            SourceCaptureValue::Enumeration(value) => {
                self.tag(6)?;
                self.declaration(value.declaration, depth)?;
                self.integer(value.value)
            }
            SourceCaptureValue::String(value) => {
                self.tag(5)?;
                self.data(value)?;
                Ok(())
            }
        }
    }
    fn binding(
        &mut self,
        binding: Binding,
        depth: usize,
    ) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        match binding {
            Binding::Declaration(id) => {
                self.tag(0)?;
                self.declaration(id, depth)
            }
            Binding::OverloadSet(id) => {
                self.tag(1)?;
                let set = self
                    .graph
                    .overload_set(id)
                    .ok_or(SourceOriginError::UnknownDeclaration)?;
                self.number(set.declarations().len() as u64)?;
                for id in set.declarations() {
                    self.step()?;
                    self.declaration(*id, depth + 1)?;
                }
                Ok(())
            }
            Binding::Module(id) => {
                self.tag(2)?;
                self.module(id, depth)
            }
            Binding::Parameter(id) => {
                self.tag(3)?;
                let parameter = self
                    .graph
                    .parameter(id)
                    .ok_or(SourceOriginError::UnboundValue)?;
                self.data(self.graph.symbols.name(parameter.name).as_bytes())?;
                self.location(parameter.location)?;
                self.tag(u8::from(parameter.program_wide))?;
                self.module(parameter.module, depth + 1)?;
                self.value(&parameter.value, depth + 1)
            }
            Binding::StorageMember(id) => {
                self.tag(4)?;
                let member = self
                    .graph
                    .source_storage_member(id)
                    .ok_or(SourceOriginError::UnknownDeclaration)?;
                self.declaration(member.owner(), depth + 1)?;
                self.number(member.path().len() as u64)?;
                for name in member.path() {
                    self.step()?;
                    self.data(self.graph.symbols.name(*name).as_bytes())?;
                }
                Ok(())
            }
            Binding::SourceMember {
                declaration,
                member,
            } => {
                self.tag(5)?;
                self.declaration(declaration, depth + 1)?;
                self.data(self.graph.symbols.name(member).as_bytes())?;
                Ok(())
            }
        }
    }
    fn source(&mut self, id: SourceId) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        let source = self
            .graph
            .sources
            .get(id)
            .ok_or(SourceOriginError::UnknownFile)?;
        self.data(source.path().as_os_str().as_encoded_bytes())?;
        Ok(())
    }
    fn source_slice(&mut self, location: SourceSpan) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
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
        self.data(relevant.as_bytes())?;
        Ok(())
    }
    fn declaration(
        &mut self,
        id: DeclarationId,
        depth: usize,
    ) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        let declaration = self
            .graph
            .declaration(id)
            .ok_or(SourceOriginError::UnknownDeclaration)?;
        self.source_slice(declaration.location())?;
        self.number(declaration.location().span.start as u64)?;
        self.number(declaration.location().span.end as u64)?;
        self.data(self.graph.symbols.name(declaration.name()).as_bytes())?;
        self.file(declaration.file, depth + 1)
    }
    fn integer(&mut self, value: Integer) -> Result<(), SourceOriginAdmissionError<E>> {
        self.tag(u8::from(value.ty().signed()))?;
        self.number(u64::from(value.ty().bits()))?;
        self.number(value.bits())?;
        Ok(())
    }
    fn float(&mut self, value: jai_types::FloatValue) -> Result<(), SourceOriginAdmissionError<E>> {
        self.tag(match value.ty() {
            FloatType::F32 => 0,
            FloatType::F64 => 1,
        })?;
        self.number(value.bits())?;
        Ok(())
    }
    fn scalar(&mut self, value: &Value) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        match value {
            Value::Literal(value) => {
                self.tag(0)?;
                self.append(&value.to_le_bytes())?;
            }
            Value::Int(value) => {
                self.tag(1)?;
                self.integer(*value)?;
            }
            Value::Bool(value) => {
                self.tag(2)?;
                self.tag(u8::from(*value))?;
            }
            Value::Float(value) => {
                self.tag(3)?;
                self.float(*value)?;
            }
            Value::WeakFloat(value) => {
                self.tag(4)?;
                let key = value
                    .request_key()
                    .canonical_bytes_with_work(usize::MAX, usize::MAX, &mut |work, bytes| {
                        (self.admit)(work, bytes)
                    })
                    .map_err(weak_error)?;
                self.data(&key)?;
            }
        }
        Ok(())
    }
    fn value(
        &mut self,
        value: &ParameterValue,
        depth: usize,
    ) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        match value {
            ParameterValue::Scalar(value) => self.scalar(value)?,
            ParameterValue::String(value) => {
                self.tag(5)?;
                self.data(value.as_bytes())?;
            }
            ParameterValue::Enumeration(value) => {
                self.tag(6)?;
                self.declaration(value.declaration, depth)?;
                self.integer(value.value)?;
            }
            ParameterValue::Type(value) => {
                self.tag(7)?;
                self.ty(value, depth)?;
            }
            ParameterValue::ContextualMember(_) => {
                return Err(SourceOriginError::UnboundValue.into());
            }
        }
        Ok(())
    }
    fn argument(
        &mut self,
        value: &ModuleBoundArgument,
        depth: usize,
    ) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        match value {
            ModuleBoundArgument::Type(value) => {
                self.tag(0)?;
                self.ty(value, depth)?;
            }
            ModuleBoundArgument::Integer(value) => {
                self.tag(1)?;
                self.integer(*value)?;
            }
            ModuleBoundArgument::Boolean(value) => {
                self.tag(2)?;
                self.tag(u8::from(*value))?;
            }
            ModuleBoundArgument::Float(value) => {
                self.tag(3)?;
                self.float(*value)?;
            }
            ModuleBoundArgument::Enumeration(value) => {
                self.tag(4)?;
                self.declaration(value.declaration, depth)?;
                self.integer(value.value)?;
            }
            ModuleBoundArgument::String(value) => {
                self.tag(5)?;
                self.data(value)?;
            }
        }
        Ok(())
    }
    fn ty(&mut self, ty: &ModuleType, depth: usize) -> Result<(), SourceOriginAdmissionError<E>> {
        self.step()?;
        if depth > 128 {
            return Err(SourceOriginError::ExcessiveNesting.into());
        }
        match ty {
            ModuleType::Builtin(ty) => {
                self.tag(0)?;
                match ty {
                    ModuleBuiltin::Scalar(ScalarType::Int(ty)) => {
                        self.tag(0)?;
                        self.tag(u8::from(ty.signed()))?;
                        self.number(u64::from(ty.bits()))?;
                    }
                    ModuleBuiltin::Scalar(ScalarType::Bool) => self.tag(1)?,
                    ModuleBuiltin::Float(FloatType::F32) => self.tag(2)?,
                    ModuleBuiltin::Float(FloatType::F64) => self.tag(3)?,
                    ModuleBuiltin::String => self.tag(4)?,
                    ModuleBuiltin::Void => self.tag(5)?,
                    ModuleBuiltin::Type => self.tag(6)?,
                    ModuleBuiltin::Context => self.tag(7)?,
                    ModuleBuiltin::Any => self.tag(8)?,
                }
            }
            ModuleType::Declaration(id) => {
                self.tag(1)?;
                self.declaration(*id, depth)?;
            }
            ModuleType::Application {
                template,
                arguments,
            } => {
                self.tag(2)?;
                self.declaration(*template, depth)?;
                self.number(arguments.len() as u64)?;
                for argument in arguments {
                    self.step()?;
                    self.argument(argument, depth + 1)?;
                }
            }
            ModuleType::Pointer(inner) => {
                self.tag(3)?;
                self.ty(inner, depth + 1)?;
            }
            ModuleType::Slice(inner) => {
                self.tag(4)?;
                self.ty(inner, depth + 1)?;
            }
            ModuleType::DynamicArray(inner) => {
                self.tag(5)?;
                self.ty(inner, depth + 1)?;
            }
            ModuleType::FixedArray {
                element,
                count,
            } => {
                self.tag(6)?;
                self.number(*count)?;
                self.ty(element, depth + 1)?;
            }
            ModuleType::Procedure(procedure) => {
                self.tag(7)?;
                self.tag(match procedure.convention {
                    CallingConvention::Jai => 0,
                    CallingConvention::C => 1,
                    CallingConvention::Stdcall => 2,
                    CallingConvention::CppMethod => 3,
                })?;
                self.tag(match procedure.return_abi {
                    jai_types::ForeignReturnAbi::Natural => 0,
                    jai_types::ForeignReturnAbi::CppNonPod => 1,
                })?;
                self.tag(match procedure.context {
                    ContextMode::Implicit => 0,
                    ContextMode::None => 1,
                })?;
                match procedure.variadic {
                    ModuleVariadic::None => self.tag(0)?,
                    ModuleVariadic::C {
                        fixed_parameters,
                    } => {
                        self.tag(1)?;
                        self.number(fixed_parameters as u64)?;
                    }
                    ModuleVariadic::Jai {
                        parameter,
                    } => {
                        self.tag(2)?;
                        self.number(parameter as u64)?;
                    }
                }
                self.number(procedure.parameters.len() as u64)?;
                for ty in &procedure.parameters {
                    self.step()?;
                    self.ty(ty, depth + 1)?;
                }
                self.number(procedure.results.len() as u64)?;
                for ty in &procedure.results {
                    self.step()?;
                    self.ty(ty, depth + 1)?;
                }
            }
        }
        Ok(())
    }
}

/// Retain every framed canonical key write; never reduce exact weak decimals to
/// a rounded float or to a collision-prone digest.
struct KeyBytes<'a, E> {
    bytes: Vec<u8>,
    admit: &'a mut dyn FnMut(usize, usize) -> Result<(), E>,
    error: Option<SourceOriginAdmissionError<E>>,
}
impl<E> KeyBytes<'_, E> {
    fn complete(self) -> Result<Vec<u8>, SourceOriginAdmissionError<E>> {
        match self.error {
            Some(error) => Err(error),
            None => Ok(self.bytes),
        }
    }
}
impl<E> Hasher for KeyBytes<'_, E> {
    fn finish(&self) -> u64 {
        0
    }
    fn write(&mut self, bytes: &[u8]) {
        if self.error.is_some() {
            return;
        }
        let result = (|| {
            debit(self.admit, bytes.len().saturating_add(1), 0)?;
            let extra = bytes
                .len()
                .checked_add(8)
                .ok_or(SourceOriginError::CapacityOverflow)?;
            reserve(&mut self.bytes, extra, self.admit)?;
            self.bytes
                .extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            self.bytes.extend_from_slice(bytes);
            Ok(())
        })();
        if let Err(error) = result {
            self.error = Some(error);
        }
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

fn debit<E>(
    admit: &mut dyn FnMut(usize, usize) -> Result<(), E>,
    work: usize,
    bytes: usize,
) -> Result<(), SourceOriginAdmissionError<E>> {
    admit(work, bytes).map_err(SourceOriginAdmissionError::Admission)
}
fn reserve<T, E>(
    values: &mut Vec<T>,
    extra: usize,
    admit: &mut dyn FnMut(usize, usize) -> Result<(), E>,
) -> Result<(), SourceOriginAdmissionError<E>> {
    let wanted = values
        .len()
        .checked_add(extra)
        .ok_or(SourceOriginError::CapacityOverflow)?;
    if wanted <= values.capacity() {
        return Ok(());
    }
    let capacity = wanted
        .max(
            values
                .capacity()
                .checked_mul(2)
                .ok_or(SourceOriginError::CapacityOverflow)?,
        )
        .max(4);
    let bytes = capacity
        .checked_add(values.capacity())
        .and_then(|n| n.checked_mul(std::mem::size_of::<T>()))
        .ok_or(SourceOriginError::CapacityOverflow)?;
    debit(admit, values.len().saturating_add(1), bytes)?;
    values
        .try_reserve_exact(capacity - values.len())
        .map_err(|_| SourceOriginError::Allocation)?;
    if values.capacity() > capacity {
        return Err(SourceOriginError::CapacityOverflow.into());
    }
    Ok(())
}
fn push<T, E>(
    values: &mut Vec<T>,
    value: T,
    admit: &mut dyn FnMut(usize, usize) -> Result<(), E>,
) -> Result<(), SourceOriginAdmissionError<E>> {
    reserve(values, 1, admit)?;
    values.push(value);
    Ok(())
}
fn weak_error<E>(
    error: jai_eval::floats::WeakFloatAdmissionError<E>,
) -> SourceOriginAdmissionError<E> {
    use jai_eval::floats::{WeakFloatAdmissionError as A, WeakFloatEncodingError as W};
    match error {
        A::Admission(error) => SourceOriginAdmissionError::Admission(error),
        A::CapacityOverflow | A::Encoding(W::ByteLimit) => {
            SourceOriginError::CapacityOverflow.into()
        }
        A::Allocation => SourceOriginError::Allocation.into(),
        A::Encoding(W::NodeLimit) => SourceOriginError::ExcessiveNesting.into(),
    }
}
