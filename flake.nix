{
  description = "Terrier";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    devenv.url = "github:cachix/devenv";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      devenv,
      rust-overlay,
      ...
    }@inputs:
    let
      devSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      packageSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = systems: f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      packages = forAllSystems packageSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        {
          default = rustPlatform.buildRustPackage {
            pname = "terrier";
            version = "0.1.0";
            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [
              dioxus-cli
              pkg-config
              wasm-bindgen-cli_0_2_108 # pinned in Cargo.lock
              binaryen # for wasm-opt
            ];

            buildInputs = with pkgs; [ openssl ];

            buildPhase = ''
              runHook preBuild
              export HOME=$(mktemp -d)
              dx bundle --platform web --release
              runHook postBuild
            '';

            dontCargoBuild = true;
            doCheck = false;

            installPhase = ''
              runHook preInstall
              mkdir -p $out/public
              cp target/dx/terrier/release/web/terrier $out/terrier
              cp -r target/dx/terrier/release/web/public/* $out/public/
              runHook postInstall
            '';
          };
        }
      );

      devShells = forAllSystems devSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            config.allowUnfree = true;
          };
        in
        {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [ ./nix/devenv.nix ];
          };
        }
      );

      nixosModules.default = import ./nix/module.nix { inherit self; };
    };
}
