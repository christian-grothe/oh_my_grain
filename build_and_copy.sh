#!/bin/bash

cargo xtask bundle granular_delay --release
cp -r target/bundled/Granular\ Delay.vst3/ ~/.vst3/
