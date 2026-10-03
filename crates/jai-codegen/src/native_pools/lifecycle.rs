//! Reset and destruction preserve owned blocks until explicit release.
use super::*;

impl<'a, 'ctx> Emitter<'a, 'ctx> {
    pub(super) fn reset_body(&self) -> Result<(), Error> {
        let (function, args) = self.start("reset");
        let owner = args[0].into_pointer_value();
        let nominal = args[1].into_pointer_value();
        let flat = args[2].into_int_value();
        let overwrite = args[3].into_int_value();
        let reserve = args[4].into_int_value();
        let default_align = args[5].into_int_value();
        let left = args[6].into_pointer_value();
        let block_slot = args[7].into_pointer_value();
        let pos_slot = args[8].into_pointer_value();
        let alignment_slot = args[9].into_pointer_value();
        self.need(self.builder.build_is_not_null(owner, "pool.nonnull")?)?;
        self.call("acquire", &[])?;
        let slot = self.state_slot(owner)?;
        let state = self.pp(slot)?;
        self.call(
            "validate",
            &[
                state.into(),
                nominal.into(),
                flat.into(),
                left.into(),
                block_slot.into(),
                pos_slot.into(),
            ],
        )?;
        let flat_align = self.bb(function, "flat.align");
        let plain_align = self.bb(function, "plain.align");
        let cursor = self.bb(function, "cursor");
        self.branch(flat, flat_align, plain_align)?;
        self.at(flat_align);
        let configured = self.ip(alignment_slot)?;
        let flat_alignment = self
            .builder
            .build_select(
                self.compare(IntPredicate::EQ, configured, self.constant(0))?,
                self.constant(8),
                configured,
                "pool.alignment",
            )?
            .into_int_value();
        self.jump(cursor)?;
        self.at(plain_align);
        self.jump(cursor)?;
        self.at(cursor);
        let alignment = self.builder.build_phi(self.integer, "pool.alignment")?;
        alignment.add_incoming(&[(&flat_alignment, flat_align), (&default_align, plain_align)]);
        let alignment = alignment.as_basic_value().into_int_value();
        self.check_alignment(alignment)?;
        let position = self.aligned(reserve, alignment)?;
        let clear = self.bb(function, "clear");
        let owned = self.bb(function, "owned");
        let done = self.bb(function, "done");
        self.branch(
            self.builder.build_is_null(state, "pool.empty")?,
            clear,
            owned,
        )?;
        self.at(owned);
        let head = self.pp(self.field(self.state, state, 3)?)?;
        let head_capacity = self.ip(self.field(self.block, head, 3)?)?;
        self.need(self.compare(IntPredicate::ULE, position, head_capacity)?)?;
        // Check every child before poisoning any block or rewinding the cursor.
        let check_walk = self.bb(function, "children.walk");
        let check = self.bb(function, "children.check");
        let checked = self.bb(function, "children.done");
        self.jump(check_walk)?;
        self.at(check_walk);
        let candidate = self.builder.build_phi(self.pointer, "pool.child.block")?;
        candidate.add_incoming(&[(&head, owned)]);
        let candidate_value = candidate.as_basic_value().into_pointer_value();
        self.branch(
            self.builder.build_is_null(candidate_value, "pool.last")?,
            checked,
            check,
        )?;
        self.at(check);
        self.call("check.children", &[state.into(), candidate_value.into()])?;
        let child_next = self.pp(candidate_value)?;
        candidate.add_incoming(&[(&child_next, check)]);
        self.jump(check_walk)?;
        self.at(checked);
        let walk = self.bb(function, "overwrite.walk");
        let fill = self.bb(function, "fill");
        let rewind = self.bb(function, "rewind");
        self.branch(overwrite, walk, rewind)?;
        self.at(walk);
        let block = self.builder.build_phi(self.pointer, "pool.block")?;
        block.add_incoming(&[(&head, checked)]);
        let block_value = block.as_basic_value().into_pointer_value();
        self.branch(
            self.builder.build_is_null(block_value, "pool.last")?,
            rewind,
            fill,
        )?;
        self.at(fill);
        let data = self.pp(self.field(self.block, block_value, 2)?)?;
        let capacity = self.ip(self.field(self.block, block_value, 3)?)?;
        self.builder.build_memset(
            data,
            1,
            self.context.i8_type().const_int(0xcc, false),
            capacity,
        )?;
        let next = self.pp(block_value)?;
        block.add_incoming(&[(&next, fill)]);
        self.jump(walk)?;
        self.at(rewind);
        self.call(
            "cursor",
            &[
                state.into(),
                head.into(),
                position.into(),
                left.into(),
                block_slot.into(),
                pos_slot.into(),
            ],
        )?;
        self.jump(done)?;
        self.at(clear);
        self.clear_cursor(left, block_slot, pos_slot)?;
        self.jump(done)?;
        self.at(done);
        self.call("unlock", &[])?;
        self.done()
    }
    fn clear_cursor(
        &self,
        left: PointerValue<'ctx>,
        block: PointerValue<'ctx>,
        position: PointerValue<'ctx>,
    ) -> Result<(), Error> {
        self.store(left, self.constant(0).into())?;
        self.store(block, self.pointer.const_null().into())?;
        self.store(position, self.constant(0).into())
    }
    pub(super) fn children_body(&self) -> Result<(), Error> {
        let (function, args) = self.start("check.children");
        let state = args[0].into_pointer_value();
        let block = args[1].into_pointer_value();
        let data = self.pp(self.field(self.block, block, 2)?)?;
        let start = self
            .builder
            .build_ptr_to_int(data, self.integer, "pool.start")?;
        let capacity = self.ip(self.field(self.block, block, 3)?)?;
        let end = self.builder.build_int_add(start, capacity, "pool.end")?;
        self.need(self.compare(IntPredicate::UGE, end, start)?)?;
        let head = self.pp(self.head.as_pointer_value())?;
        let entry = self.builder.get_insert_block().ok_or(Error::Invariant)?;
        let walk = self.bb(function, "walk");
        let check = self.bb(function, "check");
        let done = self.bb(function, "done");
        self.jump(walk)?;
        self.at(walk);
        let candidate = self.builder.build_phi(self.pointer, "pool.child")?;
        candidate.add_incoming(&[(&head, entry)]);
        let candidate_value = candidate.as_basic_value().into_pointer_value();
        self.branch(
            self.builder.build_is_null(candidate_value, "pool.empty")?,
            done,
            check,
        )?;
        self.at(check);
        let owner = self.pp(self.field(self.state, candidate_value, 1)?)?;
        let address = self
            .builder
            .build_ptr_to_int(owner, self.integer, "pool.owner.address")?;
        let inside = self.both(
            self.compare(IntPredicate::UGE, address, start)?,
            self.compare(IntPredicate::ULT, address, end)?,
        )?;
        let other = self
            .builder
            .build_not(self.same_pointer(candidate_value, state)?, "pool.other")?;
        let bad = self.both(inside, other)?;
        self.need(self.builder.build_not(bad, "pool.no.live.child")?)?;
        let next = self.pp(candidate_value)?;
        candidate.add_incoming(&[(&next, check)]);
        self.jump(walk)?;
        self.at(done);
        self.done()
    }
    pub(super) fn release_body(&self) -> Result<(), Error> {
        let (function, args) = self.start("release");
        let owner = args[0].into_pointer_value();
        let nominal = args[1].into_pointer_value();
        let flat = args[2].into_int_value();
        let left = args[3].into_pointer_value();
        let block_slot = args[4].into_pointer_value();
        let pos = args[5].into_pointer_value();
        self.need(self.builder.build_is_not_null(owner, "pool.nonnull")?)?;
        self.call("acquire", &[])?;
        let slot = self.state_slot(owner)?;
        let state = self.pp(slot)?;
        self.call(
            "validate",
            &[
                state.into(),
                nominal.into(),
                flat.into(),
                left.into(),
                block_slot.into(),
                pos.into(),
            ],
        )?;
        let clear = self.bb(function, "clear");
        let owned = self.bb(function, "owned");
        let check_walk = self.bb(function, "check.walk");
        let check = self.bb(function, "check");
        let detach = self.bb(function, "detach");
        let walk = self.bb(function, "dispose.walk");
        let dispose = self.bb(function, "dispose");
        let dispose_state = self.bb(function, "dispose.state");
        self.branch(
            self.builder.build_is_null(state, "pool.empty")?,
            clear,
            owned,
        )?;
        self.at(owned);
        let head = self.pp(self.field(self.state, state, 3)?)?;
        self.jump(check_walk)?;
        self.at(check_walk);
        let checked = self.builder.build_phi(self.pointer, "pool.block")?;
        checked.add_incoming(&[(&head, owned)]);
        let checked_value = checked.as_basic_value().into_pointer_value();
        self.branch(
            self.builder.build_is_null(checked_value, "pool.last")?,
            detach,
            check,
        )?;
        self.at(check);
        self.call("check.children", &[state.into(), checked_value.into()])?;
        let next_checked = self.pp(checked_value)?;
        checked.add_incoming(&[(&next_checked, check)]);
        self.jump(check_walk)?;
        self.at(detach);
        let successor = self.pp(state)?;
        self.store(slot, successor.into())?;
        self.jump(walk)?;
        self.at(walk);
        let block = self.builder.build_phi(self.pointer, "pool.block")?;
        block.add_incoming(&[(&head, detach)]);
        let block_value = block.as_basic_value().into_pointer_value();
        self.branch(
            self.builder.build_is_null(block_value, "pool.last")?,
            dispose_state,
            dispose,
        )?;
        self.at(dispose);
        let next = self.pp(block_value)?;
        let raw = self.pp(self.field(self.block, block_value, 1)?)?;
        self.call("free", &[raw.into()])?;
        self.call("free", &[block_value.into()])?;
        block.add_incoming(&[(&next, dispose)]);
        self.jump(walk)?;
        self.at(dispose_state);
        self.call("free", &[state.into()])?;
        self.jump(clear)?;
        self.at(clear);
        self.clear_cursor(left, block_slot, pos)?;
        self.call("unlock", &[])?;
        self.done()
    }
}
