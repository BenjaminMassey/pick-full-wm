# pick-full-wm

`pick-full-wm`: an [x11](https://en.wikipedia.org/wiki/X_Window_System) window manager written in [Rust](https://rust-lang.org/) with the [x11rb crate](https://crates.io/crates/x11rb)'s [xcb](https://xcb.freedesktop.org/) bindings.

The primary workflow is to focus on one nearly full screen window at a time, with the ability to quickly switch to other windows preview-able on the side of the screen.

The name is lightly inspired by [Minecraft](https://www.minecraft.net/en-us)'s ["Pick Block"](https://minecraft.fandom.com/wiki/Controls) key-binding that allows the user to instantly switch to any visible block.

# Demo

https://github.com/user-attachments/assets/758ea78a-3893-43a3-b48f-3e881a7a093e

# Usage

An x11 window manager is built into a single executable, so should work from a standardly built Rust file. Simply put, a `cargo build --release` should build you a working version of `pick-full-wm` at `/your/repo/target/release/pick-full-wm`.

There two primary ways (that I know of) to launch into `pick-full-wm`.

The first is via an appropriate [`.xinitrc` file from a TTY session via the [`startx`](https://manpages.debian.org/buster/xinit/startx.1.en.html): it should be placed directly at your user folder (`~` / `/home/username/`). An example `.xinitrc` is found in the root of this repository.

The second is from a display manager such as [`SDDM`](https://github.com/sddm/sddm), which looks for an appropriate [`.desktop` file](https://wiki.archlinux.org/title/Desktop_entries) in `/usr/share/xsessions/`, and can be loaded from some window manager menu (bottom left in SDDM's interface). An example `pick-full.desktop` is found in the root of this repository.

The next layer is configuring your `settings.toml` file. There is are two examples in the root of this repository: `setting-mouse.toml` and `setting-keyboard.toml`. Choose your preferred starter, copy it as "settings.toml" into `~/.config/pick-full-wm/`, and start editing as desired.

The current mouse defaults use [`xfce4-panel`](https://packages.debian.org/trixie/xfce4-panel) for both launcher and status bar. The current keyboard defaults use [`rofi`](https://github.com/davatorium/rofi) as an application launcher and [`polybar`](https://github.com/polybar/polybar) as a status bar. Both use `brightnessctl` for brightness control and `wpctl` for volume control. These are very much dynamic and change-able, though, of course: just startup and exclude clarifications.

# Features and TODOs

- [x] Splits area in "main" and "side"
- [x] "Side" area auto lays out all non-main windows
- [x] Window for "main" area swappable with left click
- [x] "Side" windows close-able with right click
- [x] Configurable `settings.toml` from user `.config` folder
- [x] Set-able list of startup applications
- [x] Set-able list of applications to exclude from managing
- [x] Set-able "launcher" application to run with Super key
- [x] Set-able size of "main" area via percent or pixels
- [x] Implementation of x11 error handling
- [x] Keyboard options for window switching
- [x] Center mouse within and focus on main window on swap
- [x] Toggle-able "full screen" mode
- [x] Customizable position of area
- [x] Keybind to open integrated help window
- [x] Keybinding visual "hints" for side windows
- [x] Work with Extended Window Manager Hints
- [x] Built-in multi-monitor support
- [x] Tiling exceptions for pop-up style windows
- [x] Some implementation of virtual workspaces
- [x] Set new windows to main space (optional)
- [x] Dynamic keybinds to run any command
- [x] Ability to move main to different workspace
- [x] Mouse-driven way to close main window
- [x] Support for close and move-workspace client messages
- [ ] Mouse driven way to move across monitors
- [ ] Ability to swap between floating and tiled
- [ ] Support for monitor rotation
- [ ] Way to move floating windows
- [ ] Logging
- [ ] Ability to move windows between monitors
- [ ] Wallpaper support
- [ ] Integration with "cargo deb"
- [ ] Support for screenshot and capture tools
- [ ] Extra support for keybind settings (like "X + Y")
- [ ] Media keys shouldn't need SUPER held

Another bigger consideration is whether to implement a kind of preview system for side windows (instead of doing true direct changes to the size and position): TBD on whether that is desired or not

# Credit

This was primarily written by Benjamin Massey benjamin.w.massey@gmail.com

With xlib (x11 crate) => xcb (x11rb crate) conversion help by Bart Massey bart@cs.pdx.edu

# License

This project is licensed under the [GNU General Public License (GPLv3)](https://www.gnu.org/licenses/gpl-3.0.en.html).

A copy of said license can be found in `LICENSE.md`.
