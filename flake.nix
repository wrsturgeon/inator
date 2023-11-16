{
  inputs = {
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };
  outputs = { fenix, flake-utils, naersk, self }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        name = "inator";
        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };
        settings = {
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
