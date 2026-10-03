// Independently authored adapter over the official LLVM demangler AST APIs.
#include <llvm/Demangle/ItaniumDemangle.h>
#include <llvm/Demangle/MicrosoftDemangle.h>
#include <algorithm>
#include <charconv>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <new>
#include <string>
#include <string_view>
#include <vector>

namespace I = llvm::itanium_demangle;
namespace M = llvm::ms_demangle;

struct Native_Type {
  int32_t kind = 99;
  int32_t bytes = 0;
  Native_Type *child = nullptr;
  const char *name = nullptr;
  int64_t extent = 0;
  Native_Type **arguments = nullptr;
  int64_t argument_count = 0;
};
struct Native_Signature {
  Native_Type *result = nullptr;
  Native_Type **arguments = nullptr;
  int64_t argument_count = 0;
  int64_t used_bytes = 0;
  int32_t status = 1;
  const char *error = "invalid mangled function";
};
static_assert(sizeof(Native_Type) == 48, "private 64-bit normalized type ABI");
static_assert(sizeof(Native_Signature) == 48, "private 64-bit signature ABI");

class Pool {
  struct Chunk { std::unique_ptr<unsigned char[]> bytes; size_t capacity; };
  std::vector<Chunk> chunks;
  size_t current = 0, offset = 0;
public:
  size_t used = 0;
  void reset() { current = offset = used = 0; }
  void *allocate(size_t size) {
    if (size > 1024 * 1024) throw std::bad_alloc();
    for (;;) {
      if (current == chunks.size()) {
        size_t capacity = std::max(size, size_t(65536));
        chunks.push_back({std::make_unique<unsigned char[]>(capacity), capacity});
      }
      size_t aligned = (offset + alignof(std::max_align_t) - 1) & ~(alignof(std::max_align_t) - 1);
      if (aligned <= chunks[current].capacity && size <= chunks[current].capacity - aligned) {
        void *result = chunks[current].bytes.get() + aligned;
        used += aligned - offset + size;
        offset = aligned + size;
        return result;
      }
      ++current;
      offset = 0;
    }
  }
  template<class T> T *make() { return new (allocate(sizeof(T))) T(); }
  Native_Type **array(size_t count) {
    if (count > 1024) throw std::bad_alloc();
    auto result = static_cast<Native_Type **>(allocate(sizeof(Native_Type *) * count));
    std::fill_n(result, count, nullptr);
    return result;
  }
  const char *copy(std::string_view name) {
    char *result = static_cast<char *>(allocate(name.size() + 1));
    std::memcpy(result, name.data(), name.size());
    result[name.size()] = 0;
    return result;
  }
};

struct Ast_Allocator {
  Pool storage;
  void reset() { storage.reset(); }
  template<class T, class... Args> T *makeNode(Args &&...args) {
    return new (storage.allocate(sizeof(T))) T(std::forward<Args>(args)...);
  }
  void *allocateNodeArray(size_t count) { return storage.allocate(count * sizeof(I::Node *)); }
};

struct Parser {
  Pool pool;
  Native_Signature signature;
  int long_bytes = 8;
  bool char_signed = true;

  Native_Type *scalar(int kind, int bytes) {
    auto type = pool.make<Native_Type>();
    type->kind = kind; type->bytes = bytes;
    return type;
  }
  Native_Type *primitive(std::string_view name) {
    if (name == "void") return scalar(0, 0);
    if (name == "bool") return scalar(1, 1);
    if (name == "char") return scalar(char_signed ? 2 : 3, 1);
    if (name == "signed char") return scalar(2, 1);
    if (name == "unsigned char" || name == "char8_t") return scalar(3, 1);
    if (name == "short") return scalar(2, 2);
    if (name == "unsigned short" || name == "char16_t") return scalar(3, 2);
    if (name == "int") return scalar(2, 4);
    if (name == "unsigned int" || name == "char32_t") return scalar(3, 4);
    if (name == "long") return scalar(2, long_bytes);
    if (name == "unsigned long") return scalar(3, long_bytes);
    if (name == "long long" || name == "__int64") return scalar(2, 8);
    if (name == "unsigned long long" || name == "unsigned __int64") return scalar(3, 8);
    if (name == "__int128") return scalar(2, 16);
    if (name == "unsigned __int128") return scalar(3, 16);
    if (name == "float") return scalar(4, 4);
    if (name == "double") return scalar(4, 8);
    if (name == "..." || name == "long double" || name == "wchar_t" || name == "decltype(nullptr)" || name == "__float128") return scalar(99, 0);
    auto type = scalar(6, 0);
    type->name = pool.copy(name);
    return type;
  }
  std::string render(const I::Node *node) {
    I::OutputBuffer output;
    node->print(output);
    std::unique_ptr<char, decltype(&std::free)> buffer(output.getBuffer(), std::free);
    return std::string(static_cast<std::string_view>(output));
  }
  Native_Type *itanium(const I::Node *node, unsigned depth = 0) {
    if (!node || depth > 64) return scalar(99, 0);
    switch (node->getKind()) {
    case I::Node::KQualType:
      return itanium(static_cast<const I::QualType *>(node)->getChild(), depth + 1);
    case I::Node::KNameType:
      return primitive(static_cast<const I::NameType *>(node)->getName());
    case I::Node::KPointerType: {
      auto type = scalar(5, 0);
      type->child = itanium(static_cast<const I::PointerType *>(node)->getPointee(), depth + 1);
      return type;
    }
    case I::Node::KReferenceType: {
      auto type = scalar(5, 0);
      static_cast<const I::ReferenceType *>(node)->match([&](const I::Node *pointee, I::ReferenceKind) {
        type->child = itanium(pointee, depth + 1);
      });
      return type;
    }
    case I::Node::KArrayType: {
      auto type = scalar(7, 0);
      type->extent = -1;
      static_cast<const I::ArrayType *>(node)->match([&](const I::Node *base, I::Node *dimension) {
        type->child = itanium(base, depth + 1);
        if (dimension) {
          auto text = render(dimension);
          auto converted = std::from_chars(text.data(), text.data() + text.size(), type->extent);
          if (converted.ec != std::errc() || converted.ptr != text.data() + text.size()) type->extent = -1;
        }
      });
      return type;
    }
    case I::Node::KFunctionType: {
      auto type = scalar(8, 0);
      static_cast<const I::FunctionType *>(node)->match([&](const I::Node *result, I::NodeArray parameters, auto, auto, auto) {
        type->child = itanium(result, depth + 1);
        type->argument_count = static_cast<int64_t>(parameters.size());
        type->arguments = pool.array(parameters.size());
        for (size_t i = 0; i < parameters.size(); ++i) type->arguments[i] = itanium(parameters[i], depth + 1);
      });
      return type;
    }
    case I::Node::KNestedName: case I::Node::KQualifiedName:
    case I::Node::KNameWithTemplateArgs: case I::Node::KGlobalQualifiedName:
      return primitive(render(node));
    default: return scalar(99, 0);
    }
  }
  Native_Type *microsoft(const M::TypeNode *node, unsigned depth = 0) {
    if (!node || depth > 64) return scalar(99, 0);
    switch (node->kind()) {
    case M::NodeKind::PrimitiveType: {
      auto kind = static_cast<const M::PrimitiveTypeNode *>(node)->PrimKind;
      if (kind == M::PrimitiveKind::Void) return scalar(0, 0);
      if (kind == M::PrimitiveKind::Bool) return scalar(1, 1);
      if (kind == M::PrimitiveKind::Char) return scalar(char_signed ? 2 : 3, 1);
      if (kind == M::PrimitiveKind::Schar) return scalar(2, 1);
      if (kind == M::PrimitiveKind::Uchar || kind == M::PrimitiveKind::Char8) return scalar(3, 1);
      if (kind == M::PrimitiveKind::Short) return scalar(2, 2);
      if (kind == M::PrimitiveKind::Ushort || kind == M::PrimitiveKind::Char16 || kind == M::PrimitiveKind::Wchar) return scalar(3, 2);
      if (kind == M::PrimitiveKind::Int || kind == M::PrimitiveKind::Long) return scalar(2, 4);
      if (kind == M::PrimitiveKind::Uint || kind == M::PrimitiveKind::Ulong || kind == M::PrimitiveKind::Char32) return scalar(3, 4);
      if (kind == M::PrimitiveKind::Int64) return scalar(2, 8);
      if (kind == M::PrimitiveKind::Uint64) return scalar(3, 8);
      if (kind == M::PrimitiveKind::Float) return scalar(4, 4);
      if (kind == M::PrimitiveKind::Double || kind == M::PrimitiveKind::Ldouble) return scalar(4, 8);
      return scalar(99, 0);
    }
    case M::NodeKind::PointerType: {
      auto pointer = static_cast<const M::PointerTypeNode *>(node);
      if (pointer->ClassParent) return scalar(99, 0);
      auto type = scalar(5, 0);
      type->child = microsoft(pointer->Pointee, depth + 1);
      return type;
    }
    case M::NodeKind::TagType: {
      auto type = scalar(6, 0);
      type->name = pool.copy(static_cast<const M::TagTypeNode *>(node)->QualifiedName->toString());
      return type;
    }
    case M::NodeKind::FunctionSignature: {
      auto function = static_cast<const M::FunctionSignatureNode *>(node);
      if (function->IsVariadic) return scalar(99, 0);
      auto type = scalar(8, 0);
      type->child = microsoft(function->ReturnType, depth + 1);
      type->argument_count = function->Params ? static_cast<int64_t>(function->Params->Count) : 0;
      type->arguments = pool.array(static_cast<size_t>(type->argument_count));
      for (int64_t i = 0; i < type->argument_count; ++i) {
        auto parameter = function->Params->Nodes[i];
        if (!M::TypeNode::classof(parameter)) return scalar(99, 0);
        type->arguments[i] = microsoft(static_cast<const M::TypeNode *>(parameter), depth + 1);
      }
      return type;
    }
    case M::NodeKind::ArrayType: {
      auto array = static_cast<const M::ArrayTypeNode *>(node);
      Native_Type *element = microsoft(array->ElementType, depth + 1);
      if (!array->Dimensions) return scalar(99, 0);
      for (size_t i = array->Dimensions->Count; i > 0; --i) {
        auto dimension = array->Dimensions->Nodes[i - 1];
        if (!M::IntegerLiteralNode::classof(dimension)) return scalar(99, 0);
        auto integer = static_cast<const M::IntegerLiteralNode *>(dimension);
        if (integer->IsNegative || integer->Value > INT64_MAX) return scalar(99, 0);
        auto type = scalar(7, 0);
        type->child = element;
        type->extent = static_cast<int64_t>(integer->Value);
        element = type;
      }
      return element;
    }
    default: return scalar(99, 0);
    }
  }
};

#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif
extern "C" EXPORT const Native_Signature *developer_check_parse(
    const char *name, int64_t count, int32_t windows, int32_t long_bytes, int32_t char_signed) {
  thread_local Parser parser;
  parser.pool.reset();
  parser.signature = Native_Signature();
  struct Usage {
    Parser &parser;
    ~Usage() { parser.signature.used_bytes = static_cast<int64_t>(parser.pool.used); }
  } usage{parser};
  parser.long_bytes = long_bytes;
  parser.char_signed = char_signed != 0;
  if (!name || count <= 0 || count > 1024 || (long_bytes != 4 && long_bytes != 8)) return &parser.signature;
  try {
    std::string_view spelling(name, static_cast<size_t>(count));
    if (windows) {
      M::Demangler demangler;
      auto symbol = demangler.parse(spelling);
      if (demangler.Error || !symbol || !spelling.empty() || !M::FunctionSymbolNode::classof(symbol)) return &parser.signature;
      auto function = static_cast<M::FunctionSymbolNode *>(symbol)->Signature;
      auto normalized = parser.microsoft(function);
      if (normalized->kind != 8) return &parser.signature;
      parser.signature.result = function->ReturnType ? normalized->child : nullptr;
      parser.signature.arguments = normalized->arguments;
      parser.signature.argument_count = normalized->argument_count;
    } else {
      if (spelling.substr(0, 3) == "__Z") spelling.remove_prefix(1);
      I::ManglingParser<Ast_Allocator> demangler(spelling.data(), spelling.data() + spelling.size());
      auto root = demangler.parse();
      if (!root || root->getKind() != I::Node::KFunctionEncoding) return &parser.signature;
      auto function = static_cast<const I::FunctionEncoding *>(root);
      if (function->getReturnType()) parser.signature.result = parser.itanium(function->getReturnType());
      auto parameters = function->getParams();
      parser.signature.argument_count = static_cast<int64_t>(parameters.size());
      parser.signature.arguments = parser.pool.array(parameters.size());
      for (size_t i = 0; i < parameters.size(); ++i) parser.signature.arguments[i] = parser.itanium(parameters[i]);
    }
    parser.signature.used_bytes = static_cast<int64_t>(parser.pool.used);
    parser.signature.status = 0;
    parser.signature.error = nullptr;
  } catch (const std::bad_alloc &) {
    parser.signature.error = "mangled-type pool allocation/size limit failure";
    parser.signature.status = 2;
  }
  return &parser.signature;
}
