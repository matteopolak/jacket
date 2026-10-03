//! Typed LLVM construction of the shared native pool ownership ledger.
use super::*;
use inkwell::{
    AddressSpace, AtomicOrdering,
    module::Linkage,
    types::{BasicType, BasicTypeEnum, PointerType, StructType},
    values::GlobalValue,
};
mod allocate;
mod lifecycle;

pub(super) fn define<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    target: &target::NativeTarget,
) -> Result<(), Error> {
    Emitter::new(context, module, target)?.define()
}

struct Emitter<'a, 'ctx> {
    context: &'ctx Context,
    module: &'a Module<'ctx>,
    builder: Builder<'ctx>,
    pointer: PointerType<'ctx>,
    integer: IntType<'ctx>,
    size: IntType<'ctx>,
    maximum: u64,
    state: StructType<'ctx>,
    block: StructType<'ctx>,
    head: GlobalValue<'ctx>,
    lock: GlobalValue<'ctx>,
    functions: HashMap<&'static str, FunctionValue<'ctx>>,
}
impl<'a, 'ctx> Emitter<'a, 'ctx> {
    fn new(
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        target: &target::NativeTarget,
    ) -> Result<Self, Error> {
        let pointer = context.ptr_type(AddressSpace::default());
        let integer = context.i64_type();
        let size = context.ptr_sized_int_type(&target.data, None);
        let maximum = match size.get_bit_width() {
            32 => u64::from(u32::MAX),
            64 => i64::MAX as u64,
            _ => return Err(Error::Invariant),
        };
        let state = context.struct_type(
            &[
                pointer.into(),
                pointer.into(),
                context.bool_type().into(),
                pointer.into(),
                pointer.into(),
                integer.into(),
                pointer.into(),
            ],
            false,
        );
        let block = context.struct_type(
            &[
                pointer.into(),
                pointer.into(),
                pointer.into(),
                integer.into(),
                integer.into(),
            ],
            false,
        );
        let head = module.add_global(pointer, None, "jai.pool.head");
        head.set_initializer(&pointer.const_null());
        head.set_linkage(Linkage::LinkOnceODR);
        head.set_visibility(inkwell::GlobalVisibility::Hidden);
        let lock = module.add_global(context.i8_type(), None, "jai.pool.lock");
        lock.set_initializer(&context.i8_type().const_zero());
        lock.set_linkage(Linkage::LinkOnceODR);
        lock.set_visibility(inkwell::GlobalVisibility::Hidden);
        let mut this = Self {
            context,
            module,
            builder: context.create_builder(),
            pointer,
            integer,
            size,
            maximum,
            state,
            block,
            head,
            lock,
            functions: HashMap::new(),
        };
        let p: BasicTypeEnum = pointer.into();
        let i: BasicTypeEnum = integer.into();
        let b: BasicTypeEnum = context.bool_type().into();
        for (name, result, parameters) in [
            ("require", None, vec![b]),
            ("acquire", None, vec![]),
            ("unlock", None, vec![]),
            ("slot", Some(p), vec![p]),
            ("validate", None, vec![p, p, b, p, p, p]),
            ("align", Some(i), vec![i, i]),
            ("cursor", None, vec![p, p, i, p, p, p]),
            ("get", Some(p), vec![p, p, b, i, i, i, p, p, p, p, p]),
            ("reset", None, vec![p, p, b, b, i, i, p, p, p, p]),
            ("check.children", None, vec![p, p]),
            ("release", None, vec![p, p, b, p, p, p]),
        ] {
            let parameters = parameters.into_iter().map(Into::into).collect::<Vec<_>>();
            let ty = match result {
                Some(result) => result.fn_type(&parameters, false),
                None => context.void_type().fn_type(&parameters, false),
            };
            let function =
                module.add_function(&format!("jai.pool.{name}"), ty, Some(Linkage::LinkOnceODR));
            this.functions.insert(name, function);
            function
                .as_global_value()
                .set_visibility(inkwell::GlobalVisibility::Hidden);
        }
        let calloc = module.get_function("calloc").unwrap_or_else(|| {
            module.add_function(
                "calloc",
                pointer.fn_type(&[size.into(), size.into()], false),
                None,
            )
        });
        let free = module.get_function("free").unwrap_or_else(|| {
            module.add_function(
                "free",
                context.void_type().fn_type(&[pointer.into()], false),
                None,
            )
        });
        if calloc.get_type() != pointer.fn_type(&[size.into(), size.into()], false)
            || free.get_type() != context.void_type().fn_type(&[pointer.into()], false)
        {
            return Err(Error::Invariant);
        }
        this.functions.insert("calloc", calloc);
        this.functions.insert("free", free);
        // The actual selected target computes metadata sizes, including padding.
        this.functions.insert(
            "metadata.state",
            module.add_function(
                "jai.pool.metadata.state",
                size.fn_type(&[], false),
                Some(Linkage::LinkOnceODR),
            ),
        );
        this.functions.insert(
            "metadata.block",
            module.add_function(
                "jai.pool.metadata.block",
                size.fn_type(&[], false),
                Some(Linkage::LinkOnceODR),
            ),
        );
        for (name, storage) in [("metadata.state", state), ("metadata.block", block)] {
            let function = this.f(name);
            this.builder
                .position_at_end(context.append_basic_block(function, "entry"));
            this.builder.build_return(Some(
                &size.const_int(target.data.get_abi_size(&storage), false),
            ))?;
        }
        Ok(this)
    }
    fn define(&self) -> Result<(), Error> {
        self.require_body()?;
        self.lock_bodies()?;
        self.slot_body()?;
        self.validate_body()?;
        self.align_body()?;
        self.cursor_body()?;
        self.get_body()?;
        self.reset_body()?;
        self.children_body()?;
        self.release_body()?;
        Ok(())
    }
    fn f(&self, name: &str) -> FunctionValue<'ctx> {
        *self.functions.get(name).expect("declared pool helper")
    }
    fn start(&self, name: &str) -> (FunctionValue<'ctx>, Vec<BasicValueEnum<'ctx>>) {
        let function = self.f(name);
        self.at(self.bb(function, "entry"));
        (function, function.get_param_iter().collect())
    }
    fn bb(&self, function: FunctionValue<'ctx>, name: &str) -> BasicBlock<'ctx> {
        self.context.append_basic_block(function, name)
    }
    fn at(&self, block: BasicBlock<'ctx>) {
        self.builder.position_at_end(block);
    }
    fn jump(&self, block: BasicBlock<'ctx>) -> Result<(), Error> {
        self.builder.build_unconditional_branch(block)?;
        Ok(())
    }
    fn branch(
        &self,
        test: IntValue<'ctx>,
        yes: BasicBlock<'ctx>,
        no: BasicBlock<'ctx>,
    ) -> Result<(), Error> {
        self.builder.build_conditional_branch(test, yes, no)?;
        Ok(())
    }
    fn done(&self) -> Result<(), Error> {
        self.builder.build_return(None)?;
        Ok(())
    }
    fn call(
        &self,
        name: &str,
        arguments: &[BasicMetadataValueEnum<'ctx>],
    ) -> Result<inkwell::values::CallSiteValue<'ctx>, Error> {
        Ok(self.builder.build_call(self.f(name), arguments, "")?)
    }
    fn need(&self, test: IntValue<'ctx>) -> Result<(), Error> {
        self.call("require", &[test.into()])?;
        Ok(())
    }
    fn compare(
        &self,
        relation: IntPredicate,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>, Error> {
        Ok(self
            .builder
            .build_int_compare(relation, left, right, "pool.check")?)
    }
    fn both(&self, left: IntValue<'ctx>, right: IntValue<'ctx>) -> Result<IntValue<'ctx>, Error> {
        Ok(self.builder.build_and(left, right, "pool.all")?)
    }
    fn same_pointer(
        &self,
        left: PointerValue<'ctx>,
        right: PointerValue<'ctx>,
    ) -> Result<IntValue<'ctx>, Error> {
        let left = self
            .builder
            .build_ptr_to_int(left, self.integer, "pool.left.address")?;
        let right = self
            .builder
            .build_ptr_to_int(right, self.integer, "pool.right.address")?;
        self.compare(IntPredicate::EQ, left, right)
    }
    fn constant(&self, value: u64) -> IntValue<'ctx> {
        self.integer.const_int(value, false)
    }
    fn field(
        &self,
        ty: StructType<'ctx>,
        pointer: PointerValue<'ctx>,
        index: u32,
    ) -> Result<PointerValue<'ctx>, Error> {
        Ok(self
            .builder
            .build_struct_gep(ty, pointer, index, "pool.slot")?)
    }
    fn load(
        &self,
        ty: BasicTypeEnum<'ctx>,
        pointer: PointerValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, Error> {
        let value = self.builder.build_load(ty, pointer, "pool.value")?;
        value
            .as_instruction_value()
            .ok_or(Error::Invariant)?
            .set_alignment(1)
            .map_err(|_| Error::Invariant)?;
        Ok(value)
    }
    fn ip(&self, pointer: PointerValue<'ctx>) -> Result<IntValue<'ctx>, Error> {
        Ok(self.load(self.integer.into(), pointer)?.into_int_value())
    }
    fn pp(&self, pointer: PointerValue<'ctx>) -> Result<PointerValue<'ctx>, Error> {
        Ok(self
            .load(self.pointer.into(), pointer)?
            .into_pointer_value())
    }
    fn store(&self, pointer: PointerValue<'ctx>, value: BasicValueEnum<'ctx>) -> Result<(), Error> {
        self.builder
            .build_store(pointer, value)?
            .set_alignment(1)
            .map_err(|_| Error::Invariant)?;
        Ok(())
    }
    fn aligned(
        &self,
        position: IntValue<'ctx>,
        alignment: IntValue<'ctx>,
    ) -> Result<IntValue<'ctx>, Error> {
        Ok(self
            .call("align", &[position.into(), alignment.into()])?
            .try_as_basic_value()
            .basic()
            .ok_or(Error::Invariant)?
            .into_int_value())
    }
    fn state_slot(&self, owner: PointerValue<'ctx>) -> Result<PointerValue<'ctx>, Error> {
        Ok(self
            .call("slot", &[owner.into()])?
            .try_as_basic_value()
            .basic()
            .ok_or(Error::Invariant)?
            .into_pointer_value())
    }
    fn allocate(&self, size: IntValue<'ctx>) -> Result<PointerValue<'ctx>, Error> {
        let pointer = self
            .call(
                "calloc",
                &[self.size.const_int(1, false).into(), size.into()],
            )?
            .try_as_basic_value()
            .basic()
            .ok_or(Error::Invariant)?
            .into_pointer_value();
        self.need(self.builder.build_is_not_null(pointer, "pool.allocated")?)?;
        Ok(pointer)
    }
    fn metadata(&self, name: &str) -> Result<IntValue<'ctx>, Error> {
        Ok(self
            .call(name, &[])?
            .try_as_basic_value()
            .basic()
            .ok_or(Error::Invariant)?
            .into_int_value())
    }
    fn require_body(&self) -> Result<(), Error> {
        let (function, args) = self.start("require");
        let done = self.bb(function, "done");
        let trap = self.bb(function, "trap");
        self.branch(args[0].into_int_value(), done, trap)?;
        self.at(trap);
        let trap_fn = Intrinsic::find("llvm.trap")
            .and_then(|i| i.get_declaration(self.module, &[]))
            .ok_or(Error::Invariant)?;
        self.builder.build_call(trap_fn, &[], "")?;
        self.builder.build_unreachable()?;
        self.at(done);
        self.done()
    }
    fn lock_bodies(&self) -> Result<(), Error> {
        let (function, _) = self.start("acquire");
        let spin = self.bb(function, "spin");
        let done = self.bb(function, "done");
        self.jump(spin)?;
        self.at(spin);
        let exchange = self.builder.build_cmpxchg(
            self.lock.as_pointer_value(),
            self.context.i8_type().const_zero(),
            self.context.i8_type().const_int(1, false),
            AtomicOrdering::SequentiallyConsistent,
            AtomicOrdering::SequentiallyConsistent,
        )?;
        let ok = self
            .builder
            .build_extract_value(exchange, 1, "pool.locked")?
            .into_int_value();
        self.branch(ok, done, spin)?;
        self.at(done);
        self.done()?;
        self.start("unlock");
        let store = self.builder.build_store(
            self.lock.as_pointer_value(),
            self.context.i8_type().const_zero(),
        )?;
        store.set_alignment(1).map_err(|_| Error::Invariant)?;
        store
            .set_atomic_ordering(AtomicOrdering::Release)
            .map_err(|_| Error::Invariant)?;
        self.done()
    }
    fn slot_body(&self) -> Result<(), Error> {
        let (function, args) = self.start("slot");
        let owner = args[0].into_pointer_value();
        let entry = self.builder.get_insert_block().ok_or(Error::Invariant)?;
        let walk = self.bb(function, "walk");
        let check = self.bb(function, "check");
        let next = self.bb(function, "next");
        let done = self.bb(function, "done");
        self.jump(walk)?;
        self.at(walk);
        let slot = self.builder.build_phi(self.pointer, "pool.slot")?;
        slot.add_incoming(&[(&self.head.as_pointer_value(), entry)]);
        let state = self.pp(slot.as_basic_value().into_pointer_value())?;
        self.branch(
            self.builder.build_is_null(state, "pool.empty")?,
            done,
            check,
        )?;
        self.at(check);
        let actual = self.pp(self.field(self.state, state, 1)?)?;
        self.branch(self.same_pointer(owner, actual)?, done, next)?;
        self.at(next);
        slot.add_incoming(&[(&state, next)]);
        self.jump(walk)?;
        self.at(done);
        self.builder.build_return(Some(&slot.as_basic_value()))?;
        Ok(())
    }
    fn validate_body(&self) -> Result<(), Error> {
        let (function, args) = self.start("validate");
        let state = args[0].into_pointer_value();
        let nominal = args[1].into_pointer_value();
        let flat = args[2].into_int_value();
        let left = self.ip(args[3].into_pointer_value())?;
        let block = self.pp(args[4].into_pointer_value())?;
        let pos = self.ip(args[5].into_pointer_value())?;
        let fresh = self.bb(function, "fresh");
        let owned = self.bb(function, "owned");
        self.branch(
            self.builder.build_is_null(state, "pool.empty")?,
            fresh,
            owned,
        )?;
        self.at(fresh);
        let a = self.compare(IntPredicate::EQ, left, self.constant(0))?;
        let b = self.builder.build_is_null(block, "pool.null")?;
        let c = self.compare(IntPredicate::EQ, pos, self.constant(0))?;
        self.need(self.both(self.both(a, b)?, c)?)?;
        self.done()?;
        self.at(owned);
        let actual_nominal = self.pp(self.field(self.state, state, 6)?)?;
        self.need(self.same_pointer(nominal, actual_nominal)?)?;
        let actual = self
            .load(
                self.context.bool_type().into(),
                self.field(self.state, state, 2)?,
            )?
            .into_int_value();
        self.need(self.compare(IntPredicate::EQ, flat, actual)?)?;
        let current = self.pp(self.field(self.state, state, 4)?)?;
        let data = self.pp(self.field(self.block, current, 2)?)?;
        let capacity = self.ip(self.field(self.block, current, 3)?)?;
        let position = self.ip(self.field(self.state, state, 5)?)?;
        let remaining = self
            .builder
            .build_int_sub(capacity, position, "pool.remaining")?;
        let a = self.compare(IntPredicate::EQ, left, remaining)?;
        let b = self.same_pointer(block, data)?;
        let c = self.compare(IntPredicate::EQ, pos, position)?;
        self.need(self.both(self.both(a, b)?, c)?)?;
        self.done()
    }
    fn align_body(&self) -> Result<(), Error> {
        let (_, args) = self.start("align");
        let pos = args[0].into_int_value();
        let align = args[1].into_int_value();
        let mask = self
            .builder
            .build_int_sub(align, self.constant(1), "pool.mask")?;
        let sum = self.builder.build_int_add(pos, mask, "pool.sum")?;
        self.need(self.compare(IntPredicate::UGE, sum, pos)?)?;
        let inverse = self.builder.build_not(mask, "pool.inverse")?;
        let result = self.builder.build_and(sum, inverse, "pool.aligned")?;
        self.builder.build_return(Some(&result))?;
        Ok(())
    }
    fn cursor_body(&self) -> Result<(), Error> {
        let (_, args) = self.start("cursor");
        let state = args[0].into_pointer_value();
        let block = args[1].into_pointer_value();
        let position = args[2].into_int_value();
        let capacity = self.ip(self.field(self.block, block, 3)?)?;
        self.need(self.compare(IntPredicate::ULE, position, capacity)?)?;
        let remaining = self
            .builder
            .build_int_sub(capacity, position, "pool.remaining")?;
        let data = self.pp(self.field(self.block, block, 2)?)?;
        self.store(self.field(self.state, state, 4)?, block.into())?;
        self.store(self.field(self.state, state, 5)?, position.into())?;
        self.store(args[3].into_pointer_value(), remaining.into())?;
        self.store(args[4].into_pointer_value(), data.into())?;
        self.store(args[5].into_pointer_value(), position.into())?;
        self.done()
    }
    fn check_alignment(&self, alignment: IntValue<'ctx>) -> Result<(), Error> {
        let positive = self.compare(IntPredicate::SGT, alignment, self.constant(0))?;
        let bounded = self.compare(
            IntPredicate::ULE,
            alignment,
            self.constant(u64::from(u32::MAX)),
        )?;
        let mask = self
            .builder
            .build_int_sub(alignment, self.constant(1), "pool.mask")?;
        let bits = self.builder.build_and(alignment, mask, "pool.bits")?;
        let power = self.compare(IntPredicate::EQ, bits, self.constant(0))?;
        self.need(self.both(self.both(positive, bounded)?, power)?)
    }
}
