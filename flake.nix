{
  description = "Babashka development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: {
        default =
          pkgs.runCommand "wlmstr"
            {
              nativeBuildInputs = [ pkgs.makeWrapper ];
              meta.mainProgram = "wlmstr";
            }
            ''
              mkdir -p $out/share/wlmstr
              cp -r ${./src} $out/share/wlmstr/src
              makeWrapper ${pkgs.babashka-unwrapped}/bin/bb $out/bin/wlmstr \
                --add-flags "--classpath $out/share/wlmstr/src -m wlmstr.core"
            '';
      });
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            babashka # bb
            clj-kondo # linter
            cljfmt # formatter
            clojure-lsp # language server
            # clojure      # JVM Clojure が必要な場合はコメントを外す
            # jdk          # 同上
          ];

          shellHook = ''
            [ "$EDITOR" = "hx" ] && { grep -qxsF '.clj-kondo/imports/' .ignore || echo '.clj-kondo/imports/' >> .ignore; }
            echo "babashka $(bb --version)"
          '';
        };
      });

      formatter = forAllSystems (pkgs: pkgs.nixpkgs-fmt);
    };
}
