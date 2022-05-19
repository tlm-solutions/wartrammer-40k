{ stdenv, lib, domain }:

stdenv.mkDerivation {
  pname = "windshield";
  version = "0.1.0";

  src = ./.;

  patchPhase = ''
    substituteInPlace script.js \
         --replace "dvb.solutions"  "${domain}"
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp -r ./* $out/bin
  '';
  meta = with lib; {
    description = "Simple website which listens to the websockets";
    homepage = "https://github.com/dump-dvb/windshield";
  };
}
