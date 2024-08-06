<div align="center">
  <h1>iNet Wireless Menu</h1>
</div>

## About

`iwmenu` allows using your menu of choice to manage the wireless network.

## Prerequisites

[iwd](https://iwd.wiki.kernel.org/) must be installed, along with one of the supported dmenu backends.

> To ensure correct icon display, please install [nerdfonts](https://www.nerdfonts.com/); their use is however optional.

### Compatibility

- [Fuzzel](https://codeberg.org/dnkl/fuzzel/)
- [Rofi](https://github.com/davatorium/rofi/)
- [Wofi](https://hg.sr.ht/~scoopta/wofi/)
- [dmenu](https://tools.suckless.org/dmenu/)

## Installation

### Build from source

Run the following commands:

```shell
git clone https://github.com/e-tho/iwmenu
cd iwmenu
cargo build --release
```

An executable file will be generated at `target/release/iwmenu`, which you can then copy to a directory in your `$PATH`.

### Nix

Add the flake as an input:

```nix
iwmenu.url = "github:e-tho/iwmenu";
```

Install the package:

```nix
environment.systemPackages = [ inputs.iwmenu.packages.${pkgs.system}.default ];
```

## Usage

Specify a dmenu backend using `-d` or `--dmenu` option.

```
iwmenu -d fuzzel
```

## License

GPLv3
