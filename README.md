# Framework RGB Fan Tool

## Overview

This is a simple rust tool to visualize music and display simple
animations onto the Framework Desktop RGB fan, as this fan does not
expose an interface in `/sys/class/leds`.

It's mainly a wrapper + some animation logic for the
[framework-system](https://github.com/FrameworkComputer/framework-system)
`framework-lib` library, as the driver communication part was already
done, but needing to run the `framework-system` binary each time you
wanted to change anything seemed like a drag.

## Installation

First, grab the repository by cloning into it, and going inside and running `cargo build -r`.

Then, copy the binary in `target/release` over to `/usr/local/bin` or some other location of your choice.

If you want to daemonize it, you might want to write something along the lines of

    [Unit]
    Description=Runs the Framework RGB fan tool
    Before=graphical.target

    [Service]
    Type=simple
    ExecStart=/usr/local/bin/framework_rgbafan smoothspin 0E81AD D4002A FFFFFF D4002A
    Restart=on-failure
    RestartSec=5
    RemainAfterExit=yes

    [Install]
    WantedBy=multi-user.target

to `/etc/systemd/system/frmwk-rgb-fan.service`, and run `sudo systemctl daemon-reload ; sudo systemctl enable --now frmwk-rgb-fan.service` 

## Usage
Since all framework rgb fans are identical, to keep things simple, and
considering that it's necessary to run as root, I just hardcoded most
of the configuration into `src/consts.rs`, so change those if you want
to use this with adjusted settings.

The first argument to the program is the modestring, the available
modes are `solid`, `blink`, `smoothspin`, `mpd`. Everything afterwards
should be any number of hex colors without the `#`. Solid mode will
only display the first color, blink mode will display the colors in
the order given. Smoothspin mode rotates a gradient wheel clockwise,
with the colors spaced with radial symmetry. In order for MPD mode to
work, ensure the following code block is in your
`~/.config/mpd.config`.


    audio_output {
        type                    "fifo"
        name                    "frameworkrgb"
        path                    "/tmp/rgb.fifo"
        format                  "44100:16:2"
    }

I've configured it to be the case that low frequency bass bands, are
cool colors, and the high frequency bands correspond to the warm
colors, with the rainbow being between them. If you want to change
these, feel free to check out `mpd_visualizer::get_freq_color`.
