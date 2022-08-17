{ naersk, src, lib, cmake, postgresql_14 }:

naersk.buildPackage {
  pname = "data-accumulator";
  version = "0.1.0";

  src = ../.;

  cargoSha256 = lib.fakeSha256;

  nativeBuildInputs = [ cmake ];
  buildInputs = [ postgresql_14 ];

  meta = with lib; {
    description = "Server which handles request from the wartrammer-40k";
    homepage = "https://github.com/dump-dvb/wartrammer-40k";
  };
}
