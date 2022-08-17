{ stdenv, lib }:

stdenv.mkDerivation {
  pname = "wartrammer-40k-frontend";
  version = "0.1.0";

  src = ../frontend/.;

  installPhase = ''
    mkdir -p $out/bin
    cp -r ./* $out/bin
  '';

  meta = with lib; {
    description = "Simple website which people use to record their wartramming effor";
    homepage = "https://github.com/dump-dvb/wartrammer-40k";
  };
}
