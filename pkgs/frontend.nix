{ stdenvNoCC, flutterPackages, lib, cacert, git, curl }:

(self:
  let
    outputHash = if self ? vendorHash then self.vendorHash else lib.fakeSha256;

		flutter = with flutterPackages; mkFlutter {
			pname = "flutter";
			version = stable.unwrapped.version;
			src = stable.unwrapped.src;
			dart = stable.dart;
      patches = stable.unwrapped.patches ++ [ ../sort_flutter_resources.patch ];
		};
  in stdenvNoCC.mkDerivation {
    name = "wartrammer-frontend";

    src = stdenvNoCC.mkDerivation {
      name = "wartrammer-frontend-fixed-output-derivation.tar.gz";
      src = ../frontend;

      nativeBuildInputs = [
				flutter
        git
        curl
      ];

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

        rm build/web/.last_build_id
      '';

      installPhase = ''
        # Build a reproducible tar, per instructions at https://reproducible-builds.org/docs/archives/
        tar --owner=0 --group=0 --numeric-owner --format=gnu \
          --sort=name --mtime="@$SOURCE_DATE_EPOCH" \
          -czf "$out" -C "build/web/" .
      '';

      GIT_SSL_CAINFO = "${cacert}/etc/ssl/certs/ca-bundle.crt";
      SSL_CERT_FILE = "${cacert}/etc/ssl/certs/ca-bundle.crt";

      impureEnvVars = lib.fetchers.proxyImpureEnvVars
        ++ [ "GIT_PROXY_COMMAND" "NIX_GIT_SSL_CAINFO" "SOCKS_SERVER" ];

      outputHashAlgo = if self ? vendorHash then null else "sha256";
      inherit outputHash;

      meta = with lib; {
        description =
          "Simple website which people use to record their wartramming effort";
        homepage = "https://github.com/dump-dvb/wartrammer-40k";
      };
    };

    phases = [ "installPhase" ];

    installPhase = ''
      mkdir $out
      tar -xvf $src -C $out
    '';
  })
