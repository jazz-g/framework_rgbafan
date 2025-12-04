# Framework RGB Fan Tool

## Overview

This is a simple rust tool to display simple animations onto the
Framework Desktop RGB fan, since OpenRGB can't handle it, and it does
not expose an interface in `/sys/class/leds`.

It's mainly a wrapper + some animation logic for the
[framework-system](https://github.com/FrameworkComputer/framework-system)
`framework-lib` library, as I wanted to keep things as simple as
possible.

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
