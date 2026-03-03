let
    pkgs = (import (builtins.fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/159d144b44185cf9ce93a9ea419580096737bd77.zip";
        # sha256 = "1lr1h35prqkd1mkmzriwlpvxcb34kmhc9dnr48gkm8hh089hifmx";
    }) { });
    stdenv = pkgs.stdenv;
in pkgs.mkShell rec {
    name = "interview";
    shellHook = ''
        source .bashrc
    '';
    buildInputs = (with pkgs; [
        bashInteractive
        rustc
        cargo
        (pkgs.python3.buildEnv.override {
            ignoreCollisions = true;
            extraLibs = with pkgs.python3.pkgs; [
                # package list: https://search.nixos.org/packages
                # be parsimonious with 3rd party dependencies; better to show off your own code than someone else's
                # ipython
                # nose
            ];
        })
    ]);
}
