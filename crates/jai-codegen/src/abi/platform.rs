//! Supported C platform ABIs are explicit rather than inferred from pointer size.
use super::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    AppleArm64,
    AppleX86_64,
    LinuxArm64,
    LinuxX86_64,
    WindowsX86_64,
    WindowsArm64,
    WebAssembly32,
    WebAssembly64,
    AndroidArm64,
    IosArm64,
}
impl Platform {
    pub fn from_triple(triple: &str) -> Result<Self, Error> {
        let arm64 = triple.starts_with("arm64-") || triple.starts_with("aarch64-");
        let x86_64 = triple.starts_with("x86_64-");
        if triple.starts_with("wasm32-") {
            return Ok(Self::WebAssembly32);
        }
        if triple.starts_with("wasm64-") {
            return Ok(Self::WebAssembly64);
        }
        if triple.contains("windows") {
            return match (x86_64, arm64) {
                (true, _) => Ok(Self::WindowsX86_64),
                (_, true) => Ok(Self::WindowsArm64),
                _ => Err(Error::UnsupportedTarget(triple.into())),
            };
        }
        if triple.contains("android") {
            return if arm64 {
                Ok(Self::AndroidArm64)
            } else {
                Err(Error::UnsupportedTarget(triple.into()))
            };
        }
        if triple.contains("apple-") && triple.contains("ios") {
            return if arm64 {
                Ok(Self::IosArm64)
            } else {
                Err(Error::UnsupportedTarget(triple.into()))
            };
        }
        let apple =
            triple.contains("apple-") && (triple.contains("darwin") || triple.contains("macos"));
        let linux = triple.contains("linux") && !triple.contains("android");
        match (apple, linux, arm64, x86_64) {
            (true, _, true, _) => Ok(Self::AppleArm64),
            (true, _, _, true) => Ok(Self::AppleX86_64),
            (_, true, true, _) => Ok(Self::LinuxArm64),
            (_, true, _, true) => Ok(Self::LinuxX86_64),
            _ => Err(Error::UnsupportedTarget(triple.into())),
        }
    }
    pub(super) fn is_sysv_x86_64(self) -> bool {
        matches!(self, Self::AppleX86_64 | Self::LinuxX86_64)
    }
    pub(super) fn is_arm64(self) -> bool {
        matches!(
            self,
            Self::AppleArm64
                | Self::LinuxArm64
                | Self::WindowsArm64
                | Self::AndroidArm64
                | Self::IosArm64
        )
    }
    pub(super) fn is_aapcs64(self) -> bool {
        matches!(self, Self::LinuxArm64 | Self::AndroidArm64)
    }
    pub(super) fn is_webassembly(self) -> bool {
        matches!(self, Self::WebAssembly32 | Self::WebAssembly64)
    }
    pub(super) fn pointer_bytes(self) -> u32 {
        if self == Self::WebAssembly32 {
            4
        } else {
            8
        }
    }
    pub(crate) fn has_proven_cpp_method_abi(self) -> bool {
        matches!(
            self,
            Self::AppleArm64
                | Self::AppleX86_64
                | Self::LinuxArm64
                | Self::LinuxX86_64
                | Self::WindowsX86_64
                | Self::WindowsArm64
                | Self::WebAssembly32
                | Self::WebAssembly64
                | Self::AndroidArm64
                | Self::IosArm64
        )
    }
}
