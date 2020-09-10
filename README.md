# Vivid

Smol utility to change digital vibrance / saturation when a program within a list starts.

Basically VibranceGUI but without the GUI and the bloat.

Only compatible with Nvidia GPUs for now, AMD support is planned (but we'll need testers!)

## Installation

* Grab the .exe in the releases
* Create a `.vivid` folder in your user directory
* Put the exe inside
* Run a `cmd`/`powershell` session and navigate to the new directory, run `vivid.exe --edit` to create and edit a new configuration file.
* Input the settings you want
* Right click on the vivid.exe file in the aforementioned folder, click on "Create a shortcut"
* Press Windows + R, type `shell:startup`
* Drag the newly created shortcut to the folder that just opened
* Double click `vivid.exe`
* Enjoy!

The installation process will be streamlined at some point (self-installing executable with a flag) but for now please bear with the tedious installation.

## Usage

```text
Vivid 0.2.0
by Mathieu Amiot / @OtaK_
Smol utility to change digital vibrance / saturation when a program within a list starts

USAGE:
    vivid.exe [FLAGS]

FLAGS:
        --amd        Bypasses GPU detection and forces to load the AMD-specific code. It can provoke errors if you don't
                     own an AMD GPU or if drivers cannot be found on your system. Warning: This is a placeholder flag
                     and will not work, as AMD GPUs are not currently supported
    -e, --edit       Launch an editor to edit the config file
    -h, --help       Prints help information
        --nvidia     Bypasses GPU detection and forces to load the NVidia-specific code. It can provoke errors if you
                     don't own an NVidia GPU or if drivers cannot be found on your system
    -V, --version    Prints version information
```

## Configuration format

The file format used is [TOML](https://toml.io/en/).

Sample structure:

```toml
# Vibrance to restore when any non-selected program comes to foreground, included explorer.exe
desktop_vibrance = 50

# Program-specific settings
[[program_settings]]
exe_name = "r5apex.exe" # Name of the program to react on
vibrance = 72 # Vibrance value in percentage to apply when this program comes to foreground.
fullscreen_only = false # Whether or not we only apply settings when the program comes to foreground in FullScreen mode

[[program_settings]]
exe_name = "your_favorite_program.exe"
vibrance = 100
fullscreen_only = true
```

## Roadmap

* [x] Docs improvements
* [ ] Shell Icon (notification area) support
* [ ] NSIS installer
* [ ] Resolution / Display mode change support

## Credits

* Mathieu Amiot / OtaK_ - Author of this program
* The nvapi-rs developers
* VibranceGUI for the inspiration

## License

Licensed under either of these:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   [https://www.apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
   [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))
