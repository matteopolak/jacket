{
  description = "jaic: an independent compiler for the Jai programming language";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    # The pinned nightly from rust-toolchain.toml (nixpkgs only ships stable Rust).
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      inherit (nixpkgs) lib;
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: lib.genAttrs systems (system: f (pkgsFor system));
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
      rustToolchainFor = pkgs: pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      packages = forAllSystems (
        pkgs:
        let
          jaic = pkgs.callPackage ./nix/jaic.nix { rustToolchain = rustToolchainFor pkgs; };
        in
        {
          inherit jaic;
          jaifmt = pkgs.callPackage ./nix/jaifmt.nix { inherit jaic; };
          default = jaic;
        }
      );

      apps = forAllSystems (
        pkgs:
        let
          packages = self.packages.${pkgs.stdenv.hostPlatform.system};
          app = drv: name: {
            type = "app";
            program = "${drv}/bin/${name}";
            meta.description = drv.meta.description;
          };
        in
        {
          default = app packages.jaic "jaic";
          jaic = app packages.jaic "jaic";
          jailsp = app packages.jaic "jailsp";
          jailint = app packages.jaic "jailint";
          jaifmt = app packages.jaifmt "jaifmt";
        }
      );

      # `pkgs.jaic` and `pkgs.jaifmt`, built from this flake's pinned nixpkgs (LLVM 22 and the
      # nightly toolchain are not in every nixpkgs release).
      overlays.default = final: _prev: {
        inherit (self.packages.${final.stdenv.hostPlatform.system}) jaic jaifmt;
      };

      checks = forAllSystems (
        pkgs:
        let
          packages = self.packages.${pkgs.stdenv.hostPlatform.system};
        in
        {
          inherit (packages) jaic jaifmt;
        }
      );

      devShells = forAllSystems (
        pkgs:
        let
          llvmPackages = pkgs.llvmPackages_22;
        in
        {
          default = pkgs.mkShell {
            packages = [
              (rustToolchainFor pkgs)
              llvmPackages.llvm
              llvmPackages.llvm.dev
              llvmPackages.clang
              (pkgs.python314 or pkgs.python3)
              pkgs.nodejs
              pkgs.libffi
              pkgs.libxml2
              pkgs.ncurses
              pkgs.zlib
              pkgs.zstd
            ];
            LLVM_SYS_221_PREFIX = "${llvmPackages.llvm.dev}";
            JAI_LIBCLANG = "${lib.getLib llvmPackages.libclang}/lib/libclang${pkgs.stdenv.hostPlatform.extensions.sharedLibrary}";
          };
        }
      );

      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}
