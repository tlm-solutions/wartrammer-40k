{ naersk, src, lib }:

naersk.buildPackage {
  pname = "data-accumulator";
  version = "0.1.0";

  src = ../.;

  cargoSha256 = lib.fakeSha256;

  nativeBuildInputs = [ ];

  meta = with lib; {
    description = "Server which handles request from the wartrammer-40k";
    homepage = "https://github.com/dump-dvb/wartrammer-40k";
  };
}
