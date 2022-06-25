## Cycle-accurate NES Emulator Written in Rust

I started writing this as a challenge for myself and a fun way to learn Rust. My initial goal was to get a few games running semi-accurately and leave it at that, but I've been so impressed with Rust's performance that I'm now aiming for cycle-accurate emulation. 

I've made an effort to not look at other peoples' emulator code while writing this.
The only resources I've really used have been the 
[NESdev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki),
[NESDev Forums](https://forums.nesdev.org/),
[Visual 6502](http://www.visual6502.org/), 
and old 6502 hardware manuals. 

The [nes-test-roms](https://github.com/christopherpow/nes-test-roms) repo has also been a big help
in confirming my emulator behaves the same way as NES hardware. 

The emulator currently runs at full speed with video, audio, and controller support (CPU, PPU, and APU implemented). 

Mappers currently supported: 
- [iNES mapper 0 / NROM](https://nesdir.github.io/mapper0.html)
- [iNES mapper 1 / MMC1](https://nesdir.github.io/mapper1.html)
- [iNES mapper 2 / UxROM](https://nesdir.github.io/mapper2.html)
- [iNES mapper 3 / CNROM](https://nesdir.github.io/mapper3.html)
- [iNES mapper 4 / MMC3](https://nesdir.github.io/mapper4.html)
- [iNES mapper 7 / AxROM](https://nesdir.github.io/mapper7.html)

These mappers cover over 1000 games!
