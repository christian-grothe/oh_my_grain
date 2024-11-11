#!/bin/bash

cargo xtask bundle oh_my_grain --release
cp -r ./target/bundled/oh_my_grain.vst3 ~/.vst3
