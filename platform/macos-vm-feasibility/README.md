# macOS Local-VM Feasibility Prototype

This directory contains the synthetic-only ADR-0087 prototype. It is not a
production analyzer backend and is not included in release artifacts.

The prototype uses Apple's public Virtualization framework to boot a pinned
Alpine ARM64 virtual kernel with an Impresari-built initramfs. The initramfs
contains only a fixed synthetic PID 1 and the exact `virtio_blk` module needed
by the pinned kernel. It has no package manager, login, interactive shell,
network configuration, repository parser, or analyzer.

The VM configuration exposes exactly:

- one serial result channel;
- one read-only 4 KiB synthetic input disk; and
- one fresh, fixed-capacity 1 MiB scratch disk.

It exposes no network adapter, host directory share, graphics, audio, input,
USB, clipboard, Rosetta, or persistent disk. Each controller invocation creates
and removes its own private job directory.

Run the explicit preparation step once, then the offline check:

```sh
./scripts/prepare-macos-vm-feasibility.sh
./scripts/check-macos-vm-feasibility.sh
```

Preparation downloads only the two exact official Alpine artifacts listed in
`guest-assets.json` and rejects either unless its size and SHA-256 match. The
guest never receives a network device.
