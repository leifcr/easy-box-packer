# Rust EasyBoxPacker

This is a direct copy of [easy-box-packer](https://github.com/alChaCC/easy-box-packer)

It doesn't do anything to fix the test failures, but implements `pack` in rust using [rutie](https://github.com/danielpclark/rutie).

In pure ruby, `pack_benchmark` takes ~22 seconds.  With the rust implementation, it is under 1 second.
