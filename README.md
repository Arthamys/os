# OS

This is my repo as I follow along the series of posts written by
Philipp Oppermann.

## Building the Kernel

The kernel being a free-standing Rust binary, it does not link against Rust's
`std` lib.  
We still reference the `core` lib, so we need to compile some parts of the
`std` lib, when we build our kernel.  
To do this, we use an unstable cargo feature:
[`build-std`](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std)
that is configured in the `.cargo/config` file.

Building the kernel is pretty simple, we build a freestanding binary that will
then be run on hardware that we emulate using [qemu](https://www.qemu.org/).

In order to create a bootimage that can be understood by qemu (the emulator
we use to virualize our kernel) we use the cargo tool `bootimage`
(`cargo install bootimage`).  
Bootimage basically combines a bootloader with out kernel image, to give us a
full bootable disk.

To run the kernel in qemu, simply run

```bash
cargo run
# Or you can invoke qemu directly:
qemu-system-x86_64 -drive format=raw,file=target/x86_64-os/debug/bootimage-os.bin
```

### Troubleshooting

If you cannot build the kernel, make sure you have the following installed:  
qemu (`yay -S qemu`)  
rust nightly (`rustup install nightly`)  
rust source code (`rustup component add rust-src`)  
llvm-tools-preview (`rustup component add llvm-tools-preview`)  
bootimage (`cargo install bootimage`)  

## Resources

* [Building an OS in Rust](https://os.phil-opp.com)
* [Operating Systems: Three Easy Pieces](http://pages.cs.wisc.edu/~remzi/OSTEP/)
* [UCSD's CSE 120 (Spring 2018 session)](https://cseweb.ucsd.edu/classes/sp18/cse120-a/)
