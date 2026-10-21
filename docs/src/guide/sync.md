# Sync between devices and with collaborators

quire keeps a library the same on your computers by connecting them directly: no server stores
your library. Connections use QUIC through [iroh](https://iroh.computer): each device is known by
a public key, connections are end-to-end encrypted, and when two devices cannot reach each other
directly (different networks, firewalls) the encrypted traffic passes through a public relay
that cannot read it.

Both devices must be online at once. For a library that is always reachable, and PDFs that wait
until a device needs them, use a [sync hub](sync-hub.md): an always-on store, run by you or by us,
that holds your library encrypted and cannot read it.

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
library joins the other's library on the first sync.

Your devices are recognised as yours by your **user key**, which must be the same on all of them.
To start on a new computer:

```bash
# on a device you already use
quire user key export          # prints the key: keep it secret
# on the new one
quire init
quire user key import <key>
quire sync peer add laptop <laptop's ID>   # and add the new device on the laptop too
quire sync with laptop
```

Two devices with the same library but different user keys refuse to sync, and say so.

## Sharing a project with someone

A project can be shared with other people, each with a role:

| role | may |
|---|---|
| `reader` | see the project, its subprojects, their references and PDFs |
| `editor` | also add, edit and unlink references, add subprojects and PDFs |
| `owner` | also add and remove members |

```bash
# your collaborator prints their user ID
quire user id
# you share the project with it
quire project share thesis <their user ID> --name Alice --role editor
quire project members thesis
quire project unshare thesis <their user ID>
```

Then pair one of their devices with one of yours (`quire sync id`, `quire sync peer add`, as
above) and sync. The project appears in their library, next to their own projects; changes go
both ways at each sync.

What a collaborator sees is the shared project only. Your tags, notes, highlights, reading
status, saved searches and your other projects stay yours, and theirs stay theirs. Each side
checks what the other sends against the sender's role: a reader's edits are refused, an editor
cannot add members, and nobody but you can put one of your references in the trash (an editor
removes a paper by unlinking it from the shared project). `quire sync with` reports refused
changes.

The first time a shared project arrives, its owner is recorded and never changes (trust on first
use, as with SSH keys).

## What happens with conflicts

References are merged field by field: when two devices (or two people) changed different fields
of the same reference, both changes are kept; when they changed the same field, the later change
wins. Time comes from a clock that never goes back and always moves past the times it has seen
from other devices; a change stamped more than a day in the future (a device with a wrong
clock) is refused. Other records (notes, tags, projects, …) take the later change as a whole.

The same paper added on two devices (the same DOI or other identifier) becomes one reference.
Two papers that got the same cite key keep theirs on one device and get a suffix on the other,
the same choice everywhere. Projects and tags with the same name are treated as one.

Sync works on the library folder; your settings (`config.toml`) are not synced.
