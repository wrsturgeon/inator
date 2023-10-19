{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = { fenix, flake-utils, naersk, nixpkgs, self }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        name = "inator";
        # pkgs = (import nixpkgs) { inherit system; };
        # naersk' = pkgs.callPackage naersk { };
        naersk' = (naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        });
        settings = {
          # cargoBuildOptions = orig: orig ++ [ "--examples" ];
          # doCheck = true;
          doDocFail = true;
          gitAllRefs = true;
          gitSubmodules = true;
          pname = "${name}";
          src = ./.;
        };
        toolchain = with fenix.packages.${system};
          combine [ minimal.cargo minimal.rustc ];
      in {
        packages = {
          ${name} = naersk'.buildPackage settings;
          default = self.packages.${system}.${name};
        };
      });
}
