# Poorkid

Poorkid is a free, open-source, work-in-progress MIDI instrument inspired by the Telepathic Instruments [Orchid](https://telepathicinstruments.com/products/orchid-limited-pre-release).

Poorkid is a Rust program that listens for numpad key presses to select chord voicings, and applies them to single incoming MIDI notes to produce chords on a new virtual MIDI output port.

The goal is to create a simple program that can be loaded onto a device like a Raspberry Pi connected to a cheap midi keyboard to make an affordable songwriting tool similar to the Orchid, (Autoharp)[https://en.wikipedia.org/wiki/Autoharp], and (Omnichord)[https://en.wikipedia.org/wiki/Omnichord].


## Disclaimer

I like the Orchid, and am very excited to receive my pre-order. I actually think it's very reasonably priced given that it's a standalone synthesizer in addition to a MIDI controller. This is mainly an exercise to learn async Rust and MIDI programming.

Also, it doesn't work yet.