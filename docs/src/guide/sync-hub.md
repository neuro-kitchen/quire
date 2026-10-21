# Sync hub: your own always-on sync

Syncing devices directly ([Sync between devices](sync.md)) needs both of them online at the same
time. A **sync hub** removes that: it is always on, your devices push their changes to it and pull
the others' whenever they like, and your PDFs wait there until a device needs them.

The hub **cannot read your library.** Every device encrypts its changes and PDFs before sending them
(XChaCha20-Poly1305, with a key only your devices have); the hub stores sealed data, your devices'
public IDs, sizes and times, nothing else. Turn the hub off and every device keeps working with what
it has; changes made meanwhile sync when it is back.

You can run the hub yourself (a container, below) or use ours (paid; coming).

## Run your own hub

You need Docker or Podman on a machine that stays on (a home server, a NAS, a small cloud server).
No port to open, no domain, no certificate: devices reach the hub by its ID over QUIC
([iroh](https://iroh.computer)), through your router.

```bash
curl -fsSLO https://raw.githubusercontent.com/neuro-kitchen/quire/main/deploy/docker-compose.yml
docker compose up -d                  # or: podman compose up -d
docker compose logs hub               # the hub's ID and a one-time setup code
```

The log shows:

```text
quire hub 0.1.0 — ID: d1e51fbfeb45e4be67631dba763049feb5cccb750a3c1d60fdb6c3ff0d24144a
Setup code (first device, once): EZYF-Y4KP-95M7
```

Settings (in a `.env` file next to `docker-compose.yml`, see `.env.example`): `QUIRE_HUB_QUOTA_MB`
(storage limit per person), `QUIRE_HUB_RELAY` (your own iroh relay), `QUIRE_HUB_VERSION`.

Everything the hub keeps is in its volume (`quire-hub-data`): back it up like any volume. It holds
only sealed data, so the backup is as private as the hub.

## Connect your devices

On your first device, with the hub's ID and the setup code:

```bash
quire sync hub add <hub-id> <setup-code>
```

The setup code works once. Then **keep your recovery kit somewhere safe**: the hub cannot read your
library, so nobody can recover it for you without your key.

```bash
quire user recovery-kit
```

Your other devices need your user key (the same person on all of them) and the hub's ID, no code:

```bash
quire user key import <key from `quire user key export` on the first device>
quire sync hub add <hub-id>
```

A new, empty library joins yours on its first sync. From then on, on any device:

```bash
quire sync hub now
```

Other people on your hub: `quire-hub invite` (in the container: `docker compose exec hub
quire-hub invite`) prints a one-time code for them; each person's library stays apart.

## PDFs on demand

By default every device downloads every PDF when it syncs. On a phone or a small laptop you can keep
only what you read:

```bash
quire config set files.download on-open   # PDFs come when opened
quire config set files.cache_mb 2048      # and at most 2 GB of them stay here
quire pdf keep thesis --project           # this project's PDFs: always here
quire pdf keep rivera2025                 # this paper's too
```

Opening a paper (`quire pdf open`, or the app) looks for its PDF here, then on the hub, then on your
other devices, then at the source (open access, the publisher) as when it was first added. Past the
cache limit, the least recently opened PDFs are removed from the device, but **only those the hub
holds**: a device never removes what may be the last copy, and without a hub nothing is removed.
"Holds" is what the last sync found: a PDF the hub has lost and has no room to take again (over
quota) stays on the device.

## Devices

```bash
quire sync hub devices           # your devices the hub knows
quire sync hub revoke <device>   # a lost phone: the hub refuses it from now on
quire sync hub rm                # this device stops using the hub
```

A revoked device still has what it had downloaded, and the keys it held: revoking stops it syncing,
it does not erase it.
