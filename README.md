<div align="center">
  <h1>iwmenu</h1>
</div>

## About

`iwmenu` (**i**Net **W**ireless **Menu**) allows using your menu of choice to manage the wireless network.

## Prerequisites

[iwd](https://iwd.wiki.kernel.org/) must be installed, along with either a supported launcher or any `stdin` mode launcher.

> [!NOTE]
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

```shell
git clone https://github.com/e-tho/iwmenu
cd iwmenu
cargo build --release
```

An executable file will be generated at `target/release/iwmenu`, which you can then copy to a directory in your `$PATH`.

### Nix

Add the flake as an input:

```nix
inputs.iwmenu.url = "github:e-tho/iwmenu";
```

Install the package:

```nix
{ inputs, ... }:
{
  environment.systemPackages = [ inputs.iwmenu.packages.${pkgs.system}.default ];
}
```

### Arch Linux

Install the package with your favorite AUR helper:

```shell
paru -S iwmenu-git
```

## Usage

### Supported menus

Specify an application using `-m` or `--menu` flag.

```shell
iwmenu -m fuzzel
```

### Custom menus

Specify `custom` as the menu and set your command using the `--menu-command` flag. Ensure your launcher supports `stdin` mode, and that it is properly configured in the command.

```shell
iwmenu -m custom --menu-command "my_custom_launcher --flag"
```

#### Prompt and Placeholder support

Use either `{prompt}` or `{placeholder}` as the value for the relevant flag in your command; each will be replaced with the appropriate text as needed. They return the same string, with `{prompt}` adding a colon at the end.

```shell
iwmenu -m custom --menu-command "my_custom_launcher --prompt-flag '{prompt}'" # or --placeholder-flag '{placeholder}'
```

#### Password obfuscation support

To enable support for password obfuscation, set the appropriate flag via `{password_flag:--my-password-flag}`.

```shell
iwmenu -m custom --menu-command "my_custom_launcher {password_flag:--my-password-flag}"
```

#### Example to enable all features

This example demonstrates enabling all available features in custom mode with `fuzzel`.

```shell
iwmenu -m custom --menu-command "fuzzel -d -p '{prompt}' {password_flag:--password}"
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
