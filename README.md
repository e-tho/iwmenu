<div align="center">
  <h1>iNet Wireless Menu</h1>
</div>

## About

`iwmenu` allows using your menu of choice to manage the wireless network.

## Prerequisites

[iwd](https://iwd.wiki.kernel.org/) must be installed, along with one of the supported dmenu backends.

> To ensure proper icon display, you can either install [nerdfonts](https://www.nerdfonts.com/) for font-based icons (usage is optional) or use the `--icon xdg` flag for image-based icons from your XDG theme.

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

Specify a dmenu backend using `-d` or `--dmenu` flag.

```
iwmenu -d fuzzel
```

### Available Options

| Flag             | Description                                         | Supported Values                  | Default Value |
| ---------------- | --------------------------------------------------- | --------------------------------- | ------------- |
| `-d`, `--dmenu`  | Specify the dmenu backend to use.                   | `dmenu`, `rofi`, `wofi`, `fuzzel` | `dmenu`       |
| `-i`, `--icon`   | Specify the icon type to use.                       | `font`, `xdg`                     | `font`        |
| `-s`, `--spaces` | Specify icon to text space count (font icons only). | Any positive integer              | `1`           |

## License

GPLv3
