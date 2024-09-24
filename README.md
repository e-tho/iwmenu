<div align="center">
  <h1>iNet Wireless Menu</h1>
</div>

## About

`iwmenu` allows using your menu of choice to manage the wireless network.

## Prerequisites

[iwd](https://iwd.wiki.kernel.org/) must be installed, along with one of the supported launchers.

> To ensure proper icon display, you can either install [nerdfonts](https://www.nerdfonts.com/) for font-based icons (usage is optional) or use the `--icon xdg` flag for image-based icons from your XDG theme.

### Compatibility

- [Fuzzel](https://codeberg.org/dnkl/fuzzel/)
- [Rofi](https://github.com/davatorium/rofi/)
- [Wofi](https://hg.sr.ht/~scoopta/wofi/)
- [dmenu](https://tools.suckless.org/dmenu/)

Use `custom` mode if your launcher is not supported.

## Installation

### Build from source

Run the following commands:

```console
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

### Arch Linux

Install the package with your favorite AUR helper:

```console
paru -S iwmenu-git
```

## Usage

Specify an application using `-m` or `--menu` flag.

```console
iwmenu -m fuzzel
```

If your launcher is not supported, or you need to add additional flags, you can specify `custom` as the menu and provide your command using the `--menu-command` flag. Ensure your launcher supports an input/script mode, and that it is properly configured in the command.

```console
iwmenu -m custom --menu-command "my_custom_launcher --flag"
```

To enable prompt support in custom menus, use `{prompt}` as the value for the relevant flag in your command. This way, when a prompt is required, it will be replaced with the appropriate text.

```console
iwmenu -m custom --menu-command "my_custom_launcher --prompt-flag '{prompt}'"
```

To enable support for password obfuscation, set the appropriate flag via `{password_flag:--my-password-flag}`.

```console
iwmenu -m custom --menu-command "my_custom_launcher {password_flag:--my-password-flag}"
```

### Available Options

| Flag             | Description                                           | Supported Values                            | Default Value |
| ---------------- | ----------------------------------------------------- | ------------------------------------------- | ------------- |
| `-m`, `--menu`   | Specify the menu application to use.                  | `dmenu`, `rofi`, `wofi`, `fuzzel`, `custom` | `dmenu`       |
| `--menu-command` | Specify the command to use when `custom` menu is set. | Any valid shell command                     | `None`        |
| `-i`, `--icon`   | Specify the icon type to use.                         | `font`, `xdg`                               | `font`        |
| `-s`, `--spaces` | Specify icon to text space count (font icons only).   | Any positive integer                        | `1`           |

## License

GPLv3
