//! Checked block selection and target-sized native allocation.
use super::*;

impl<'a, 'ctx> Emitter<'a, 'ctx> {
    pub(super) fn get_body(&self) -> Result<(), Error> {
        let (function, args) = self.start("get");
        let owner = args[0].into_pointer_value();
        let nominal = args[1].into_pointer_value();
        let flat = args[2].into_int_value();
        let size = args[3].into_int_value();
        let reserve = args[4].into_int_value();
        let default_align = args[5].into_int_value();
        let capacity_slot = args[6].into_pointer_value();
        let left_slot = args[7].into_pointer_value();
        let block_slot = args[8].into_pointer_value();
        let pos_slot = args[9].into_pointer_value();
        let alignment_slot = args[10].into_pointer_value();
        self.need(self.compare(IntPredicate::SGE, size, self.constant(0))?)?;
        let zero = self.bb(function, "zero");
        let begin = self.bb(function, "begin");
        self.branch(
            self.compare(IntPredicate::EQ, size, self.constant(0))?,
            zero,
            begin,
        )?;
        self.at(zero);
        self.builder
            .build_return(Some(&self.pointer.const_null()))?;
        self.at(begin);
        self.need(self.builder.build_is_not_null(owner, "pool.nonnull")?)?;
        self.call("acquire", &[])?;
        let configured_capacity = self.ip(capacity_slot)?;
        self.need(self.compare(IntPredicate::SGE, configured_capacity, self.constant(0))?)?;
        let capacity = self
            .builder
            .build_select(
                self.compare(IntPredicate::EQ, configured_capacity, self.constant(0))?,
                self.constant(65536),
                configured_capacity,
                "pool.capacity",
            )?
            .into_int_value();
        let flat_align = self.bb(function, "flat.align");
        let plain_align = self.bb(function, "plain.align");
        let configuration = self.bb(function, "configuration");
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
        self.jump(configuration)?;
        self.at(plain_align);
        self.jump(configuration)?;
        self.at(configuration);
        let alignment = self.builder.build_phi(self.integer, "pool.alignment")?;
        alignment.add_incoming(&[(&flat_alignment, flat_align), (&default_align, plain_align)]);
        let alignment = alignment.as_basic_value().into_int_value();
        self.check_alignment(alignment)?;
        let first_position = self.aligned(reserve, alignment)?;
        let minimum = self
            .builder
            .build_int_add(first_position, size, "pool.minimum")?;
        self.need(self.compare(IntPredicate::SGE, minimum, size)?)?;
        let new_capacity = self
            .builder
            .build_select(
                self.compare(IntPredicate::UGT, minimum, capacity)?,
                minimum,
                capacity,
                "pool.new.capacity",
            )?
            .into_int_value();
        self.need(self.compare(IntPredicate::ULE, new_capacity, self.constant(self.maximum))?)?;
        let slot = self.state_slot(owner)?;
        let found = self.pp(slot)?;
        self.call(
            "validate",
            &[
                found.into(),
                nominal.into(),
                flat.into(),
                left_slot.into(),
                block_slot.into(),
                pos_slot.into(),
            ],
        )?;
        let new_state = self.bb(function, "new.state");
        let existing_state = self.bb(function, "existing.state");
        let ready = self.bb(function, "state.ready");
        self.branch(
            self.builder.build_is_null(found, "pool.fresh")?,
            new_state,
            existing_state,
        )?;
        self.at(new_state);
        let allocated = self.allocate(self.metadata("metadata.state")?)?;
        self.store(self.field(self.state, allocated, 1)?, owner.into())?;
        self.store(self.field(self.state, allocated, 6)?, nominal.into())?;
        self.store(self.field(self.state, allocated, 2)?, flat.into())?;
        self.store(slot, allocated.into())?;
        self.jump(ready)?;
        self.at(existing_state);
        self.jump(ready)?;
        self.at(ready);
        let state = self.builder.build_phi(self.pointer, "pool.state")?;
        state.add_incoming(&[(&allocated, new_state), (&found, existing_state)]);
        let state = state.as_basic_value().into_pointer_value();
        let head_slot = self.field(self.state, state, 3)?;
        let cur_slot = self.field(self.state, state, 4)?;
        let current = self.pp(cur_slot)?;
        let position = self.ip(self.field(self.state, state, 5)?)?;
        let walk = self.bb(function, "walk");
        let check = self.bb(function, "check.block");
        let advance = self.bb(function, "advance");
        let new_block = self.bb(function, "new.block");
        let selected = self.bb(function, "selected");
        self.jump(walk)?;
        self.at(walk);
        let block = self.builder.build_phi(self.pointer, "pool.block")?;
        block.add_incoming(&[(&current, ready)]);
        let previous = self.builder.build_phi(self.pointer, "pool.previous")?;
        previous.add_incoming(&[(&self.pointer.const_null(), ready)]);
        let candidate_position = self.builder.build_phi(self.integer, "pool.position")?;
        candidate_position.add_incoming(&[(&position, ready)]);
        let block_value = block.as_basic_value().into_pointer_value();
        self.branch(
            self.builder.build_is_null(block_value, "pool.empty")?,
            new_block,
            check,
        )?;
        self.at(check);
        let start = self.aligned(
            candidate_position.as_basic_value().into_int_value(),
            alignment,
        )?;
        let end = self.builder.build_int_add(start, size, "pool.end")?;
        self.need(self.compare(IntPredicate::SGE, end, size)?)?;
        let available = self.ip(self.field(self.block, block_value, 3)?)?;
        let guarantee = self.ip(self.field(self.block, block_value, 4)?)?;
        self.branch(
            self.both(
                self.compare(IntPredicate::ULE, end, available)?,
                self.compare(IntPredicate::UGE, guarantee, alignment)?,
            )?,
            selected,
            advance,
        )?;
        self.at(advance);
        let next = self.pp(block_value)?;
        block.add_incoming(&[(&next, advance)]);
        previous.add_incoming(&[(&block_value, advance)]);
        candidate_position.add_incoming(&[(&reserve, advance)]);
        self.jump(walk)?;
        self.at(new_block);
        let mask = self
            .builder
            .build_int_sub(alignment, self.constant(1), "pool.mask")?;
        let total = self
            .builder
            .build_int_add(new_capacity, mask, "pool.total")?;
        self.need(self.both(
            self.compare(IntPredicate::UGE, total, new_capacity)?,
            self.compare(IntPredicate::ULE, total, self.constant(self.maximum))?,
        )?)?;
        let total_size = self
            .builder
            .build_int_cast(total, self.size, "pool.size_t")?;
        let raw = self.allocate(total_size)?;
        let address = self
            .builder
            .build_ptr_to_int(raw, self.integer, "pool.address")?;
        let data_address = self.aligned(address, alignment)?;
        let data = self
            .builder
            .build_int_to_ptr(data_address, self.pointer, "pool.data")?;
        let created = self.allocate(self.metadata("metadata.block")?)?;
        self.store(self.field(self.block, created, 1)?, raw.into())?;
        self.store(self.field(self.block, created, 2)?, data.into())?;
        self.store(self.field(self.block, created, 3)?, new_capacity.into())?;
        self.store(self.field(self.block, created, 4)?, alignment.into())?;
        let previous = previous.as_basic_value().into_pointer_value();
        let append_slot = self
            .builder
            .build_select(
                self.builder.build_is_null(previous, "pool.first")?,
                head_slot,
                previous,
                "pool.append",
            )?
            .into_pointer_value();
        self.store(append_slot, created.into())?;
        self.jump(selected)?;
        self.at(selected);
        let chosen = self.builder.build_phi(self.pointer, "pool.chosen")?;
        chosen.add_incoming(&[(&block_value, check), (&created, new_block)]);
        let chosen = chosen.as_basic_value().into_pointer_value();
        let chosen_start = self.builder.build_phi(self.integer, "pool.start")?;
        chosen_start.add_incoming(&[(&start, check), (&first_position, new_block)]);
        let chosen_start = chosen_start.as_basic_value().into_int_value();
        let chosen_end = self
            .builder
            .build_int_add(chosen_start, size, "pool.consumed")?;
        self.call(
            "cursor",
            &[
                state.into(),
                chosen.into(),
                chosen_end.into(),
                left_slot.into(),
                block_slot.into(),
                pos_slot.into(),
            ],
        )?;
        self.store(capacity_slot, capacity.into())?;
        let save_alignment = self.bb(function, "save.alignment");
        let done = self.bb(function, "done");
        self.branch(flat, save_alignment, done)?;
        self.at(save_alignment);
        self.store(alignment_slot, alignment.into())?;
        self.jump(done)?;
        self.at(done);
        let chosen_data = self.pp(self.field(self.block, chosen, 2)?)?;
        let result = jai_llvm::gep(
            &self.builder,
            self.context.i8_type().into(),
            chosen_data,
            &[chosen_start],
            "pool.result",
        )?;
        self.call("unlock", &[])?;
        self.builder.build_return(Some(&result))?;
        Ok(())
    }
}
