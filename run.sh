#!/bin/sh

cargo b --target x86_64-pc-windows-gnu
wine target/x86_64-pc-windows-gnu/debug/rq.exe $@
