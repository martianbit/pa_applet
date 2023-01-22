#!/bin/bash

cargo build --release
install -m 755 target/release/pa_applet ~/.local/bin/pa_applet

