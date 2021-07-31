#!/usr/bin/env sh
cbindgen --lang C --cpp-compat src/ffi.rs > vcs-classic-hid.h
