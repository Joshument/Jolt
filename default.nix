with import <nixpkgs> {};

(pkgs.stdenvAdapters.useMoldLinker stdenv).mkDerivation {
  name = "revolt-backend-libs";
  buildInputs = [ openssl openssl.dev ];
  nativeBuildInputs = [ pkg-config ];
  # LD_LIBRARY_PATH = lib.makeLibraryPath [ openssl ];
}
