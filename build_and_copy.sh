#!/bin/bash

cargo xtask bundle oh_my_grain --release
cp -r ./target/bundled/oh-my-grain.vst3 ~/.vst3
