{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell
{
  nativeBuildInputs = with pkgs; [
    cmake
    clang

    glfw
    #opengl
    libGL
    #wayland
    wayland
    #x11
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXinerama
    xorg.libXrandr

    #rust
    rustc
    cargo
    rustfmt

    #VS codium with extensions
    (vscode-with-extensions.override {
      vscode = vscodium;
      vscodeExtensions = with vscode-extensions; [
        rust-lang.rust-analyzer
        vadimcn.vscode-lldb
      ];
    })
  ];

  LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
    glfw
    #opengl
    libGL
    #wayland
    wayland
    #x11
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXinerama
    xorg.libXrandr
  ];

  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

  shellHook = ''
    echo "Dev environment"

    alias crd='cargo run'
    alias crr='cargo run --release'
    alias cbd='cargo build'
    alias cbr='cargo build --release'
  '';
}
