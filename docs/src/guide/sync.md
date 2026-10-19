# Sync between devices

quire keeps a library the same on your computers by connecting them directly: no server stores
your library. Connections use QUIC through [iroh](https://iroh.computer): each device is known by
a public key, connections are end-to-end encrypted, and when two devices cannot reach each other
directly (different networks, firewalls) the encrypted traffic passes through a public relay
that cannot read it.

## Pairing two devices

On each device:

```bash
quire sync id                  # prints this device's ID
```

Then tell each device about the other:

```bash
# on the laptop
quire sync peer add desktop <desktop's ID>
# on the desktop
quire sync peer add laptop <laptop's ID>
```

A device only accepts connections from the peers you added.

## Syncing

One device waits, the other connects:

```bash
# on the desktop
quire sync serve               # Ctrl-C stops
# on the laptop
quire sync with desktop
```

Both sides send what changed since they last synced, and the PDFs the other lacks. A new, empty
library joins the other's library on the first sync, so to start on a new computer: `quire init`,
pair, `quire sync with <device>`.

## What happens with conflicts

Changes are kept per row (a reference, a note, a tag on a reference, …). When both devices
changed the same row, the later change wins. When both added the same paper, or two papers got
the same cite key, the result is the same on both devices: one of the cite keys gets a suffix,
and duplicates show in `quire ref duplicates` to merge. Projects and tags with the same name are
treated as one.

Sync works on the library folder; your settings (`config.toml`) are not synced.
