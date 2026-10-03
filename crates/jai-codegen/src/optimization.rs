//! Typed optimization selection mapped once at the LLVM boundary.
use inkwell::{module::Module, passes::PassBuilderOptions, targets::TargetMachine};
pub use jai_types::{BitcodeOptimization, MachineOptimization};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Optimization {
    pub bitcode: BitcodeOptimization,
    pub machine: MachineOptimization,
}
impl Optimization {
    pub fn pipeline(self) -> &'static str {
        match self.bitcode {
            BitcodeOptimization::Unset | BitcodeOptimization::O0 => "default<O0>",
            BitcodeOptimization::O1 => "default<O1>",
            BitcodeOptimization::O2 => "default<O2>",
            BitcodeOptimization::O3 => "default<O3>",
            BitcodeOptimization::Os => "default<Os>",
            BitcodeOptimization::Oz => "default<Oz>",
        }
    }
    pub fn machine_level(self) -> inkwell::OptimizationLevel {
        use inkwell::OptimizationLevel as Llvm;
        match self.machine {
            MachineOptimization::None => Llvm::None,
            MachineOptimization::Less => Llvm::Less,
            MachineOptimization::Default => Llvm::Default,
            MachineOptimization::Aggressive => Llvm::Aggressive,
            MachineOptimization::Unset => match self.bitcode {
                BitcodeOptimization::Unset | BitcodeOptimization::O0 => Llvm::None,
                BitcodeOptimization::O1 => Llvm::Less,
                BitcodeOptimization::O3 => Llvm::Aggressive,
                BitcodeOptimization::O2 | BitcodeOptimization::Os | BitcodeOptimization::Oz => {
                    Llvm::Default
                }
            },
        }
    }
    pub fn apply(self, module: &Module<'_>, machine: &TargetMachine) -> Result<(), String> {
        module.verify().map_err(|error| error.to_string())?;
        module
            .run_passes(self.pipeline(), machine, PassBuilderOptions::create())
            .map_err(|error| error.to_string())?;
        jai_llvm::normalize_suppressed_debug(module);
        module.verify().map_err(|error| error.to_string())
    }
}
