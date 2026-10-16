// Independent consumer of the installed public LLVM output-buffer SDK.
// Outputs are caller-owned data files; this program consumes no WASM module.
#include <cstring>
#include <llvm/Support/Error.h>
#include <llvm/Support/FileOutputBuffer.h>
#include <llvm/Support/raw_ostream.h>
#include <utility>

int main(int argc, char **argv) {
  if (argc != 5)
    return 2;
  const unsigned modes[] = {
      0, llvm::FileOutputBuffer::F_mmap, llvm::FileOutputBuffer::F_executable,
      llvm::FileOutputBuffer::F_mmap | llvm::FileOutputBuffer::F_executable};
  for (int index = 0; index < 4; ++index) {
    unsigned flags = modes[index];
    auto created = llvm::FileOutputBuffer::create(argv[index + 1], 4096, flags);
    if (!created) {
      llvm::logAllUnhandledErrors(created.takeError(), llvm::errs(),
                                  "create: ");
      return 1;
    }
    auto buffer = std::move(*created);
    std::memset(buffer->getBufferStart(), 0x49, buffer->getBufferSize());
    if (auto error = buffer->commit()) {
      llvm::logAllUnhandledErrors(std::move(error), llvm::errs(), "commit: ");
      return 1;
    }
  }
  return 0;
}
