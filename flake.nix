{
description = "App";

inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
};

outputs = { self, nixpkgs }:
    let
        system = "x86_64-linux";
        pkgs = import nixpkgs {
            inherit system;
        };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        commonBuildInputs = [
            pkgs.sdl3

            pkgs.libGL

            pkgs.wayland
            pkgs.libxkbcommon

            pkgs.libx11
            pkgs.libxext
            pkgs.libxcursor
            pkgs.libxrandr
            pkgs.libxi
            pkgs.libXinerama
        ];

        commonNativeBuildInputs = [
            pkgs.cmake
            pkgs.gcc
            pkgs.clang
            pkgs.pkg-config

            pkgs.cargo
            pkgs.rustc
            pkgs.rustfmt
            pkgs.clippy
        ];
    in {
        devShells.${system} = {
            default = pkgs.mkShell {
                packages = commonBuildInputs ++ commonNativeBuildInputs;

                LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath commonBuildInputs;
                LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

                shellHook = ''

                '';
            };
        };
    };
}
