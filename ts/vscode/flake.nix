{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    let
      overlays.default = final: prev: {
        flams-vsix = prev.callPackage ./vsix.nix { };
        vscode-extensions = prev.vscode-extensions // {
          flams.flams-ext = vsixToCodeExtension prev final.flams-vsix;
        };
      };

      vsixToCodeExtension =
        pkgs: vsix:
        pkgs.vscode-utils.buildVscodeExtension {
          inherit (vsix) pname version;
          src = vsix;
          unpackPhase = "unzip $src";

          vscodeExtPublisher = vsix.packageJson.publisher;
          vscodeExtName = vsix.packageJson.name;
          vscodeExtUniqueId = "${vsix.packageJson.publisher}.${vsix.packageJson.name}";
        };

      flake = flake-utils.lib.eachDefaultSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            config.allowUnfree = true;
            overlays = [
              self.overlays.default
            ];
          };
        in
        {
          packages = {
            inherit (pkgs) flams-vsix;
            inherit (pkgs.vscode-extensions.flams) flams-ext;
            default = pkgs.flams-vsix;
          };

          apps = {
            # Builds the extension and packages is with vscode to launch it.
            # To execute with: nix run .#code
            # You can also execute it with the latest version in the repo: nix run github:sysdiglabs/vscode-extension#code
            # Or even from a tag: nix run github:sysdiglabs/vscode-extension/0.2.6#code
            code =
              let
                vscode-with-extension-installed = pkgs.vscode-with-extensions.override {
                  vscodeExtensions = with (pkgs.vscode-extensions); [ flams-vsix ];
                };
              in
              {
                type = "app";
                program = "${vscode-with-extension-installed}/bin/code";
              };
          };

          devShells.default =
            with pkgs;
            mkShell {
              shellHook = ''
                npm ci
                pre-commit install
              '';
              packages =
                [
                  vscode
                  nodejs
                  typescript
                  vsce
                  pre-commit
                  just
                  nodePackages.typescript-language-server
                  prefetch-npm-deps
                  sd
                ]
                ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
                  xvfb-run
                ];
            };

          formatter = pkgs.nixfmt-rfc-style;
        }
      );
    in
    flake // { inherit overlays; };
}
