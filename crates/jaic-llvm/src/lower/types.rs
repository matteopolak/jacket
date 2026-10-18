//! LLVM types for IR types, signatures and their attributes.
use super::Backend;
use super::plan::{Lowered, ParamAttr, ParamPlan, RetPlan};
use inkwell::AddressSpace;
use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::types::{
    AnyType, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FloatType, FunctionType, PointerType,
};
use jaic::abi::{self, Arch, Passing, PieceTy};
use jaic::ir::{Conv, Sig, Ty, WideFloat};

impl<'ctx, 'p> Backend<'ctx, 'p> {
    pub(super) fn ptr_ty(&self) -> PointerType<'ctx> {
        self.ctx.ptr_type(AddressSpace::default())
    }

    pub(super) fn ll(&self, ty: Ty) -> BasicTypeEnum<'ctx> {
        match ty {
            Ty::I8 => self.ctx.i8_type().into(),
            Ty::I16 => self.ctx.i16_type().into(),
            Ty::I32 => self.ctx.i32_type().into(),
            Ty::I64 => self.ctx.i64_type().into(),
            Ty::F32 => self.ctx.f32_type().into(),
            Ty::F64 => self.ctx.f64_type().into(),
            Ty::Ptr => self.ptr_ty().into(),
            Ty::F80 => self.ctx.x86_f80_type().into(),
            Ty::F128 => self.ctx.f128_type().into(),
        }
    }

    /// The LLVM type of a `long double` format.
    pub(super) fn wide_ty(&self, fmt: WideFloat) -> FloatType<'ctx> {
        match fmt {
            WideFloat::X87 => self.ctx.x86_f80_type(),
            WideFloat::Binary128 => self.ctx.f128_type(),
        }
    }

    pub(super) fn piece_ty(&self, ty: PieceTy) -> BasicTypeEnum<'ctx> {
        match ty {
            PieceTy::I64 => self.ctx.i64_type().into(),
            PieceTy::F32 => self.ctx.f32_type().into(),
            PieceTy::F64 => self.ctx.f64_type().into(),
            PieceTy::V2F32 => self.ctx.f32_type().vec_type(2).into(),
            PieceTy::X87 => self.ctx.x86_f80_type().into(),
            PieceTy::F128 => self.ctx.f128_type().into(),
        }
    }

    pub(super) fn bytes_ty(&self, size: u64) -> BasicTypeEnum<'ctx> {
        self.ctx.i8_type().array_type(size as u32).into()
    }

    /// Function type and parameter mapping for an IR signature.
    pub(super) fn lower_sig(&self, sig: &Sig) -> Lowered<'ctx> {
        let cabi = if sig.conv == Conv::C {
            sig.c_abi.as_deref()
        } else {
            None
        };
        let ret_agg = cabi.and_then(|c| c.ret.clone());
        let mut llvm_params: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::new();
        let mut attrs = Vec::new();
        // `#cpp_return_type_is_non_pod`: C++ returns a class with a non-trivial copy or
        // destructor through the hidden result pointer, however small it is.
        let forced_sret = cabi.is_some_and(|c| c.ret_indirect);
        let ret = match &ret_agg {
            Some(layout) => match abi::classify_ret(self.arch, layout).filter(|_| !forced_sret) {
                Some(pieces) => RetPlan::Registers(layout.clone(), pieces),
                None => {
                    attrs.push((0, ParamAttr::Sret(layout.size)));
                    if forced_sret && self.arch == Arch::Win64Arm {
                        // MSVC on arm64 passes this pointer in x0, not x8; `inreg` selects it.
                        attrs.push((0, ParamAttr::InReg));
                    }
                    llvm_params.push(self.ptr_ty().into());
                    RetPlan::Sret
                }
            },
            None => RetPlan::Scalars(sig.returns.clone()),
        };
        let mut params = Vec::new();
        let mut budget = abi::RegBudget::new(
            self.arch,
            matches!(ret, RetPlan::Sret)
                && (self.arch.is_x86_64() || forced_sret && self.arch == Arch::Win64Arm),
        );
        // Variadic arguments are not part of the LLVM function type.
        let mut fixed = None;
        for (i, &ty) in sig.params.iter().enumerate() {
            if sig.c_varargs && i == sig.c_fixed as usize {
                fixed = Some(llvm_params.len());
            }
            if ret_agg.is_some() && i == sig.out_index() {
                params.push(ParamPlan::Dropped);
                continue;
            }
            let layout = cabi.and_then(|c| c.params.get(i).cloned().flatten());
            match layout {
                Some(layout) => {
                    let passing = if sig.c_varargs {
                        abi::classify_vararg(self.arch, &layout)
                    } else {
                        abi::classify_arg(self.arch, &layout)
                    };
                    let passing = budget.aggregate(passing);
                    // Win64Arm's variadic arguments straddle registers and the stack.
                    let array = self.arch.is_aarch64()
                        && !(sig.c_varargs && self.arch == Arch::Win64Arm)
                        && matches!(&passing, Passing::Registers(p) if p.len() > 1);
                    match &passing {
                        Passing::Registers(pieces) if array => {
                            let elem = self.piece_ty(pieces[0].ty);
                            llvm_params.push(elem.array_type(pieces.len() as u32).into());
                            params.push(ParamPlan::AggArray(layout, pieces.clone()));
                            continue;
                        }
                        Passing::Registers(pieces) => {
                            for p in pieces {
                                llvm_params.push(self.piece_ty(p.ty).into());
                            }
                        }
                        Passing::ByVal => {
                            attrs.push((
                                llvm_params.len() as u32,
                                ParamAttr::ByVal(layout.size, layout.align.max(8)),
                            ));
                            llvm_params.push(self.ptr_ty().into());
                        }
                        Passing::Indirect => llvm_params.push(self.ptr_ty().into()),
                    }
                    params.push(ParamPlan::Agg(layout, passing));
                }
                None => {
                    budget.scalar(ty);
                    llvm_params.push(self.ll(ty).into());
                    params.push(ParamPlan::Scalar(ty));
                }
            }
        }
        let fixed_params = &llvm_params[..fixed.unwrap_or(llvm_params.len())];
        let fn_ty = match &ret {
            RetPlan::Scalars(tys) => self.fn_type(self.ret_type(tys), fixed_params, sig.c_varargs),
            RetPlan::Registers(_, pieces) => {
                let tys: Vec<BasicTypeEnum> = pieces.iter().map(|p| self.piece_ty(p.ty)).collect();
                self.fn_type(self.ret_type_of(&tys), fixed_params, sig.c_varargs)
            }
            RetPlan::Sret => self.fn_type(None, fixed_params, sig.c_varargs),
        };
        Lowered {
            fn_ty,
            params,
            ret,
            attrs,
        }
    }

    pub(super) fn fn_type(
        &self,
        ret: Option<BasicTypeEnum<'ctx>>,
        params: &[BasicMetadataTypeEnum<'ctx>],
        varargs: bool,
    ) -> FunctionType<'ctx> {
        match ret {
            Some(t) => t.fn_type(params, varargs),
            None => self.ctx.void_type().fn_type(params, varargs),
        }
    }

    fn ret_type(&self, tys: &[Ty]) -> Option<BasicTypeEnum<'ctx>> {
        let tys: Vec<BasicTypeEnum> = tys.iter().map(|&t| self.ll(t)).collect();
        self.ret_type_of(&tys)
    }

    fn ret_type_of(&self, tys: &[BasicTypeEnum<'ctx>]) -> Option<BasicTypeEnum<'ctx>> {
        match tys {
            [] => None,
            [one] => Some(*one),
            many => Some(self.ctx.struct_type(many, false).into()),
        }
    }

    pub(super) fn apply_attrs(
        &self,
        lowered: &Lowered<'ctx>,
        mut add: impl FnMut(AttributeLoc, Attribute),
    ) {
        for &(index, attr) in &lowered.attrs {
            let (name, size, align) = match attr {
                ParamAttr::Sret(size) => ("sret", size, 0),
                ParamAttr::ByVal(size, align) => ("byval", size, align),
                ParamAttr::InReg => {
                    let kind = Attribute::get_named_enum_kind_id("inreg");
                    add(
                        AttributeLoc::Param(index),
                        self.ctx.create_enum_attribute(kind, 0),
                    );
                    continue;
                }
            };
            let kind = Attribute::get_named_enum_kind_id(name);
            let ty = self.bytes_ty(size).as_any_type_enum();
            add(
                AttributeLoc::Param(index),
                self.ctx.create_type_attribute(kind, ty),
            );
            if align > 0 {
                let kind = Attribute::get_named_enum_kind_id("align");
                add(
                    AttributeLoc::Param(index),
                    self.ctx.create_enum_attribute(kind, align),
                );
            }
        }
    }
}
