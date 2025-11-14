{
  buildNpmPackage,
  pkg-config,
  vsce, 
}:
let
  packageJson = with builtins; fromJSON (readFile ./package.json);
in
buildNpmPackage{
  pname = "${packageJson.name}-vsix";
  version = packageJson.version;
  src = ./.;
  npmDepsHash = "";

  nativeBuildInputs = [
    pkg-config
    vsce
  ];
  dontNpmbuild = true;
  dontnpmInstall = true;
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
