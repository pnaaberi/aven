{
  description = "Prebuilt aven release binary";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      # Version and archive hashes of the release this flake is published with.
      release = builtins.fromJSON (builtins.readFile ./release.json);
      artifacts = {
        x86_64-linux = "aven-linux-amd64";
        aarch64-linux = "aven-linux-arm64";
        x86_64-darwin = "aven-darwin-amd64";
        aarch64-darwin = "aven-darwin-arm64";
      };
      systems = builtins.attrNames artifacts;
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.stdenvNoCC.mkDerivation {
            pname = "aven";
            version = release.version;

            src = pkgs.fetchurl {
              url = "https://github.com/raine/aven/releases/download/v${release.version}/${artifacts.${system}}.tar.gz";
              sha256 = release.sha256.${system};
            };
            sourceRoot = ".";

            dontConfigure = true;
            dontBuild = true;
            # Linux binaries are static and macOS binaries are signed; leave both untouched.
            dontStrip = true;
            dontPatchELF = true;

            installPhase = ''
              runHook preInstall
              install -Dm755 aven $out/bin/aven
              runHook postInstall
            '';

            meta = with pkgs.lib; {
              description = "Local-first task manager CLI and sync server";
              homepage = "https://github.com/raine/aven";
              license = licenses.mit;
              mainProgram = "aven";
              sourceProvenance = [ sourceTypes.binaryNativeCode ];
              platforms = systems;
            };
          };
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/aven";
        };
      });
    };
}
