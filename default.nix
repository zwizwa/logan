with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "lars-current";
  buildInputs = [ rustc ];
}

