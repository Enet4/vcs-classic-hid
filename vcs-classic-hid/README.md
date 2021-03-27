# VCS Classic HID

[![Latest Version](https://img.shields.io/crates/v/vcs-classic-hid.svg)](https://crates.io/crates/vcs-classic-hid) [![Build Status](https://travis-ci.org/Enet4/vcs-classic-hid-rs.svg?branch=master)](https://travis-ci.org/Enet4/vcs-classic-hid) [![dependency status](https://deps.rs/repo/github/Enet4/vcs-classic-hid-rs/status.svg)](https://deps.rs/repo/github/Enet4/vcs-classic-hid)

A specialized library for access to the Atari VCS Classic Controller.

This crate uses the [hidapi](https://crates.io/crates/hidapi)
for finding the device and opening HID access to them.
With this crate, an assortment of facilities are provided
for reading the current state of the device and,
more importantly, send force feedback and LED manipulation messages.

## Building

To build just this package:

```sh
cargo build --release -p vcs-classic-hid 
```

## Using

Light up your controller:

```rust
let device = vcs_classic_hid::open()?;
device.send(vcs_classic_hid::LedReport::filled(0xFF))?;
```

Apply some force feedback three times:

```rust
let device = vcs_classic_hid::open()?;
device.send(vcs_classic_hid::FfReport::new_with_params(
    0xA0, 30, 30, 3
))?;
```

Please see the documentation for more details.

**Cargo features:**

- `linux-hidraw` (Linux only): use HIDRAW to access the controller
- `linux-libusb` (Linux only): access the controller via libusb

## License and Warning Note

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

This work is not affiliated with Atari.

As the software and firmware of the Atari VCS system
is practically still in beta,
it is unclear whether there are, or will be, any side effects
from the prolonged use of operations to the classic controller,
be it via this library or others.
Although `vcs-classic-hid` was developed with the best effort
from the original author to make it safe to use,
it is unfeasible to make a complete assurance that it is and will always be,
completely safe for the device,
regardless of which capabilities from the library are used.

`vcs-classic-hid` is to be used at the user's own risk.
As defined by the aforementioned license,
authors and contributors to `vcs-classic-hid` and/or associated programs
cannot be held liable for any damage
which may occur from the use of this software.
