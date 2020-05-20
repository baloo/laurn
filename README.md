# Laurn

Run a dev-environment in a pure-ish nix environment.

Laurn will read your `laurn.nix` file and when running `laurn shell` you will get your project directory mounted in a namespace where only your project directory and your declared dependencies are available.
The purpose is to isolate your system from your developement environment:
 - Dependencies declaration is "pure", what's not declared is not available.
 - No libraries can extract secrets from your host (npm tokens, ssh keys, ...).

## Usage

`laurn shell`
