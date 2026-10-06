# jaifmt, the Jai formatter (jaifmt/main.jai), compiled by the jaic package.
{
  lib,
  stdenv,
  jaic,
}:
stdenv.mkDerivation {
  pname = "jaifmt";
  inherit (jaic) version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = ../jaifmt/main.jai;
  };

  nativeBuildInputs = [ jaic ];

  buildPhase = ''
    runHook preBuild
    # The source directory is named jaifmt too, so the binary is written inside it.
    jaic build jaifmt/main.jai -O2 --no-debug-info -o jaifmt/jaifmt
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 jaifmt/jaifmt $out/bin/jaifmt
    runHook postInstall
  '';

  doInstallCheck = true;
  installCheckPhase = ''
    runHook preInstallCheck
    printf 'main :: ()  {\nx:=1;\n}\n' | $out/bin/jaifmt --stdin | grep -q 'x := 1;'
    runHook postInstallCheck
  '';

  meta = {
    description = "Code formatter for Jai";
    homepage = "https://github.com/matteopolak/jai";
    mainProgram = "jaifmt";
    inherit (jaic.meta) platforms;
  };
}
