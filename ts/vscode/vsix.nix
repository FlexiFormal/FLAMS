{
  buildNpmPackage,
  pkg-config,
  vsce,
  libsecret,
  stdenv
}:
let
  packageJson = with builtins; fromJSON (readFile ./package.json);
in
buildNpmPackage{
  pname = "${packageJson.name}-vsix";
  version = packageJson.version;
  src = ./.;
  npmDepsHash = "sha256-NDKF0MfOGHU06vNA0kkxEBZg+7o5CaeuTYN6hfILkzQ=";

  nativeBuildInputs = [
    pkg-config
    vsce
  ];
  dontNpmbuild = true;
  dontnpmInstall = true;
  buildInputs = [
    libsecret
  ];
  buildPhase = ''
  vsce package
  '';
  installPhase = ''
  install -Dm444 *.vsix $out
'';
  passthru = {
   inherit packageJson; 
  };
}
