{ stdenv
, flutter
, lib
, cacert
, git
, curl
}:

stdenv.mkDerivation {
  name = "frontend";
  src = ../frontend;

  nativeBuildInputs = [ flutter git curl ];

  phases = [ "buildPhase" "installPhase" ];

  buildPhase = ''
    TMP=$(mktemp -d)

    export HOME="$TMP"
    export PUB_CACHE=''${PUB_CACHE:-"$HOME/.pub-cache"}
    export ANDROID_EMULATOR_USE_SYSTEM_LIBS=1

    flutter config --no-analytics &>/dev/null # mute first-run
    flutter config --enable-web
    mkdir src
    cp -Pr $src/* src
    chmod +w src -R
    cd src
    ls -laF
    ln -s ${flutter}/bin/cache/dart-sdk/ $HOME/.cache/flutter/dart-sdk

    # here we download the fonts so our users dont have to do this in their browser
    mkdir fonts
    curl -o fonts/Roboto.ttf https://fonts.gstatic.com/s/roboto/v20/KFOmCnqEu92Fr1Me5WZLCzYlKw.ttf
cat >> pubspec.yaml << 'EOF'
  fonts:
    - family: Roboto
      fonts:
        - asset: fonts/Roboto.ttf
EOF

    flutter pub get
    # make flutter user loacl canvaskit
    flutter build web --release -v --dart-define=FLUTTER_WEB_CANVASKIT_URL=/canvaskit/
  '';

  installPhase = ''
    mkdir $out
    cp -r build/web/* $out
  '';

  GIT_SSL_CAINFO = "${cacert}/etc/ssl/certs/ca-bundle.crt";
  SSL_CERT_FILE = "${cacert}/etc/ssl/certs/ca-bundle.crt";

  impureEnvVars = lib.fetchers.proxyImpureEnvVars ++ [
    "GIT_PROXY_COMMAND" "NIX_GIT_SSL_CAINFO" "SOCKS_SERVER"
  ];

  outputHashMode = "recursive";
  outputHash = "sha256-oo3wk6WK+FdA01UaiVbWAlnEaJrdULXTFGuuH4IMZ4o=";

  meta = with lib; {
    description = "Simple website which people use to record their wartramming effort";
    homepage = "https://github.com/dump-dvb/wartrammer-40k";
  };
}
