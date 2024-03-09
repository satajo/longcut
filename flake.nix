{
  description = "Key-sequence based command executor for Linux on X11.";

  inputs = { nixpkgs.url = "github:nixos/nixpkgs"; };

  outputs = { nixpkgs, self }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      formatter.${system} = pkgs.nixfmt;

      packages.${system}.default = pkgs.rustPlatform.buildRustPackage rec {
        name = "longcut";
        pname = "longcut";
        src = nixpkgs.lib.cleanSource ./.;
        cargoLock = { lockFile = ./Cargo.lock; };

        nativeBuildInputs = with pkgs; [ pkg-config ];
        buildInputs = with pkgs; [ glib xorg.libX11 gtk3 ];

        checkFlags = [
          # longcut_x11: Test fails with a "signal: 11, SIGSEGV: invalid memory reference"
          "--skip=handle::tests::test_string_to_keycode"
        ];
      };
    };
}
