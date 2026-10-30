# shellcheck shell=bash
# Source this: exports LLVM_VERSION, LLVM_MAJOR and LLVM_SYS_PREFIX_VAR (the name of the variable
# llvm-sys reads) derived from the `llvm` pin in .github/pins.json (tools/ci/pins.py llvm-env).
_llvm_env_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
while IFS='=' read -r _llvm_env_name _llvm_env_value; do
  export "$_llvm_env_name=$_llvm_env_value"
done < <(PYTHONPATH="$_llvm_env_dir/.." python3 "$_llvm_env_dir/pins.py" llvm-env)
unset _llvm_env_dir _llvm_env_name _llvm_env_value
