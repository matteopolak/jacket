# jaic, jailsp and jailint, built with the LLVM 23 backend and installed next to `stdlib/`
# the way the release archives lay them out (see docs/tools/nix.md).
{
  lib,
  stdenv,
  makeRustPlatform,
  makeWrapper,
  rustToolchain,
  llvmPackages_23,
  libffi,
  libxml2,
  ncurses,
  zlib,
  zstd,
}:
let
  llvmPackages = llvmPackages_23;
  rustPlatform = makeRustPlatform {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };
  # What `jaic` runs: `cc` as the linker driver, `ar` for static libraries and, on
  # macOS, `dsymutil` for debug information. Appended to PATH so a user's own `cc` wins.
  toolPath = lib.makeBinPath (
    [
      stdenv.cc
      stdenv.cc.bintools
    ]
    ++ lib.optional stdenv.hostPlatform.isDarwin llvmPackages.llvm
  );
  libclang = "${lib.getLib llvmPackages.libclang}/lib/libclang${stdenv.hostPlatform.extensions.sharedLibrary}";
in
rustPlatform.buildRustPackage {
  pname = "jaic";
  version = (lib.importTOML ../Cargo.toml).workspace.package.version;

  # Only what the build and the smoke test read. `.cargo/config.toml` is left out: its
  # crate publish-age policy guards dependency updates and needs crates.io, while this
  # build is offline against the locked, vendored crates.
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../crates
      ../stdlib
      ../examples
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
    # inkwell is a git dependency pinned to a full commit hash in Cargo.lock (LLVM 23 support
    # predates its next crates.io release); `builtins.fetchGit` fetches that exact revision,
    # so no separate output hash has to be kept in sync with the lock file.
    allowBuiltinFetchGit = true;
  };
  cargoBuildFlags = [
    "-p"
    "jaic-cli"
    "-p"
    "jai-language-server"
    "-p"
    "jailint"
  ];
  # The workspace tests need the repository's test corpus and host tools; the install
  # check below exercises the packaged compiler instead.
  doCheck = false;
  # cargo-auditable is built for nixpkgs' stable Cargo, not the pinned nightly.
  auditable = false;

  nativeBuildInputs = [
    llvmPackages.llvm.dev
    makeWrapper
  ];
  buildInputs = [
    llvmPackages.llvm
    libffi
    libxml2
    ncurses
    zlib
    zstd
  ];
  # llvm-sys runs `$LLVM_SYS_231_PREFIX/bin/llvm-config`.
  env.LLVM_SYS_231_PREFIX = "${llvmPackages.llvm.dev}";

  # jaic looks for its standard library in `stdlib/` next to the executable, and
  # its bootstrap is `stdlib/prelude/module.jai`, so binaries live beside the library.
  # bin/ holds wrappers, which exec the real path, so that lookup still works.
  postInstall = ''
    dir=$out/libexec/jaic
    mkdir -p $dir
    mv $out/bin/jaic $out/bin/jailsp $out/bin/jailint $dir/
    cp -R stdlib $dir/
    makeWrapper $dir/jaic $out/bin/jaic \
      --set-default JAI_LIBCLANG ${libclang} \
      --suffix PATH : ${toolPath}
    makeWrapper $dir/jailsp $out/bin/jailsp
    makeWrapper $dir/jailint $out/bin/jailint
  '';

  # Run and natively build two programs with the installed wrapper, outside the source tree.
  doInstallCheck = true;
  installCheckPhase = ''
    runHook preInstallCheck
    work=$(mktemp -d)
    cp examples/compile-time-record.jai $work/record.jai
    printf '#import "Basic";\nmain :: () { print("hello from nix\\n"); }\n' > $work/hello.jai
    pushd $work
    status=0; $out/bin/jaic record.jai -interp || status=$?
    test "$status" = 42
    $out/bin/jaic hello.jai -interp | grep -qx 'hello from nix'
    $out/bin/jailint -D warnings hello.jai
    $out/bin/jaic hello.jai -exe hello
    ./hello | grep -qx 'hello from nix'
    $out/bin/jaic record.jai -exe record
    status=0; ./record || status=$?
    test "$status" = 42
    popd
    runHook postInstallCheck
  '';

  passthru = { inherit llvmPackages; };

  meta = {
    description = "Independent compiler for the Jai programming language";
    homepage = "https://github.com/matteopolak/jai";
    mainProgram = "jaic";
    platforms = lib.platforms.linux ++ lib.platforms.darwin;
  };
}
