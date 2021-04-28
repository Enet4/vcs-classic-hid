# VCS Classic HID

[![Latest Version](https://img.shields.io/crates/v/vcs-classic-hid.svg)](https://crates.io/crates/vcs-classic-hid) [![dependency status](https://deps.rs/repo/github/Enet4/vcs-classic-hid/status.svg)](https://deps.rs/repo/github/Enet4/vcs-classic-hid)

A specialized library for access to the Atari VCS Classic Controller.

Check out a showcase video of the library in action [here](https://www.youtube.com/watch?v=bpBjXCxH0Sw).

This crate uses the [hidapi](https://crates.io/crates/hidapi)
for finding connected VCS classic joysticks and opening HID access to them.
With this crate, an assortment of facilities are provided
for reading the current state of the device and,
more importantly, send force feedback and LED manipulation messages.

The library can be found in the [vcs-classic-hid](vcs-classic-hid) directory.
The remaining directories are examples and other utilities.

## More

The [simulator](simulator) is a crate
that enables developers to create a simulated façade
over the classic controller.
Programs can choose to interface with the generic `Device` trait
and programmatically operate on the simulated device's state,
so as to test and develop for the classic controller
without the actual hardware.

## Examples

You will also find examples of use in the following folders:

- [simon](simon): the classic game of Simon Says!
- [cat-mouse](cat-mouse): a game of cat and mouse; catch the cheese while avoiding the cat!

To run an example program:

```sh
cargo run --release --bin vcs-classic-hid-cat-mouse
cargo run --release --bin vcs-classic-hid-simon
```

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
