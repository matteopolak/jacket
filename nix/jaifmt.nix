# jaifmt, the Jai formatter (tools/jaifmt/main.jai), compiled by the jaic package.
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
    fileset = ../tools/jaifmt/main.jai;
  };

  nativeBuildInputs = [ jaic ];

  buildPhase = ''
    runHook preBuild
    jaic build tools/jaifmt/main.jai -O2 --no-debug-info -o jaifmt
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 jaifmt $out/bin/jaifmt
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
