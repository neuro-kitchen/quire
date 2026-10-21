# Self-hosting the hub

The [sync hub](sync-hub.md) is one small program, `quire-hub`, and its data folder. This chapter
runs it on a machine of yours: a home server, a NAS, a small cloud server, any computer that stays
on. It needs **no open port, no domain and no certificate**: devices reach the hub by its ID over
QUIC ([iroh](https://iroh.computer)), through routers and firewalls.

What the hub keeps is sealed by your devices (it cannot read your library), so the machine does not
need to be trusted with your papers. It does need to be backed up: see [Backups](#backups).

Every command here was run with Podman 5.8 (rootless) on Linux, on 2026-10-21. The Docker
commands are the same with `docker` in place of `podman`; Compose was Docker Compose v2 in both
cases (Podman runs it for `podman compose`).

## The image

The hub runs from a container image of about 56 MB (the program on a minimal, non-root base with no
shell).

**Build it** from the repository. This works today, and is what you need until a published image
exists:

```bash
git clone https://github.com/neuro-kitchen/quire
cd quire
podman build -f deploy/hub/Dockerfile -t localhost/quire-hub:local .   # or: docker build …
```

The build compiles the hub inside the container: a few minutes (172 s here). On a computer you are
using meanwhile, `--env CARGO_BUILD_JOBS=4` keeps half the cores free. (Rootless Podman cannot use
`--cpuset-cpus`: the CPU controller is not delegated to users.)

**Or use the published image**, `ghcr.io/neuro-kitchen/quire-hub`, once a release of the hub is
tagged; the compose file below uses it unless told otherwise.

## Start it with Compose

Copy `deploy/docker-compose.yml` and `deploy/.env.example` (as `.env`) into a folder of their own,
for example `~/quire-hub/`. To run the image you built, set in `.env`:

```bash
QUIRE_HUB_IMAGE=localhost/quire-hub
QUIRE_HUB_VERSION=local
```

**With Podman**, `podman compose` hands the work to Docker Compose, which talks to Podman through
its socket. Turn the socket on once (as your user; no root):

```bash
systemctl --user enable --now podman.socket
```

Without it, `podman compose up` fails with *Cannot connect to the Docker daemon at
unix:///run/user/1000/podman/podman.sock*.

Then, in the folder:

```bash
podman compose up -d          # or: docker compose up -d
podman compose logs hub
```

The log shows the hub's ID and, the first time, a one-time setup code with the command to type:

```text
quire hub 0.1.0 — ID: 3c1cad153e81ac0e2de250c8c921560758742299a51a21e19f3a97990bc8d6e4
Setup code (first device, once): ATNJ-3E7P-9HDH
In quire: `quire sync hub add 3c1cad153e81ac0e2de250c8c921560758742299a51a21e19f3a97990bc8d6e4 ATNJ-3E7P-9HDH`
```

On your first device, run that line:

```text
$ quire sync hub add 3c1cad15…d6e4 ATNJ-3E7P-9HDH
connecting to the hub…
    synced through the hub  1 change(s) sent, 0 received; registered
```

The setup code then stops working, and after a restart the log shows only the ID. Connecting your
other devices, and what to keep safe, is in [Connect your devices](sync-hub.md#connect-your-devices).

`podman ps` shows the container's health (`Up 17 seconds (healthy)`): the image checks itself every
30 seconds (`quire-hub health`).

Settings, in `.env`:

| Setting | Default | Does |
|---|---|---|
| `QUIRE_HUB_IMAGE`, `QUIRE_HUB_VERSION` | the published image, `latest` | which image runs |
| `QUIRE_HUB_QUOTA_MB` | `0` (none) | storage limit of each new person, in MB |
| `QUIRE_HUB_RELAY` | empty (public relays) | your own iroh relay, for devices that cannot connect directly |
| `QUIRE_HUB_MAX_SESSIONS` | `64` | sessions served at once |
| `RUST_LOG` | `quire_hub=info,warn` | how much the log says (`quire_hub=debug` for more) |

Devices on the same network as the hub connect directly when it shares the host's network: uncomment
`network_mode: host` in `docker-compose.yml`. Without it they still connect, through hole punching
or a relay.

## Or one command, without Compose

```bash
podman run -d --name quire-hub --restart unless-stopped \
  -v quire-hub-data:/data -e QUIRE_HUB_QUOTA_MB=0 \
  localhost/quire-hub:local
podman logs quire-hub
```

## Start at boot

**Docker** starts the container again after a reboot by itself (`restart: unless-stopped`).

**Podman** has no daemon to do that: let systemd run the hub. Put this in
`~/.config/containers/systemd/quire-hub.container` (a Quadlet unit):

```ini
[Unit]
Description=quire sync hub

[Container]
Image=localhost/quire-hub:local
Volume=quire-hub-data:/data
Environment=QUIRE_HUB_QUOTA_MB=0
HealthCmd=/usr/local/bin/quire-hub health
HealthInterval=30s

[Service]
Restart=always

[Install]
WantedBy=default.target
```

```bash
systemctl --user daemon-reload
systemctl --user start quire-hub
journalctl --user -u quire-hub          # the log: ID and setup code
loginctl enable-linger "$USER"          # keep it running when you are not logged in
```

Use this or Compose for one hub, not both. `loginctl enable-linger` was not run on the test machine
(it changes how the machine treats your account); without it, user services stop at logout.

## Without containers

The hub is a single binary. Build it with Rust (see [Install](../install.md)):

```bash
cargo install --path crates/quire-hub      # into ~/.cargo/bin
quire-hub --data ~/quire-hub-data serve    # prints the ID and setup code
quire-hub --data ~/quire-hub-data id
quire-hub --data ~/quire-hub-data health   # exit code 0: working
```

`--data` (or `QUIRE_HUB_DATA`) is the hub's folder; every other setting is the same as in `.env`,
as an environment variable or option (`quire-hub serve --help`). As a system service, for example
`/etc/systemd/system/quire-hub.service` (written for this chapter, not run on the test machine,
which would need root):

```ini
[Unit]
Description=quire sync hub
After=network-online.target
Wants=network-online.target

[Service]
ExecStart=/usr/local/bin/quire-hub serve
Environment=QUIRE_HUB_DATA=/var/lib/quire-hub
DynamicUser=yes
StateDirectory=quire-hub
StateDirectoryMode=0700
Restart=always

[Install]
WantedBy=multi-user.target
```

## People and storage

Everyone on the hub has an account of their own; libraries stay apart.

```bash
podman compose exec hub quire-hub invite     # a one-time code for another person, valid 7 days
podman compose exec hub quire-hub id         # the hub's ID again
```

The other person enters `quire sync hub add <hub-id> <invitation>` on their device. A code works
once: used again it gives *this code is wrong, used or expired*. `QUIRE_HUB_QUOTA_MB` limits each
new account; a device over its limit still syncs its changes and keeps the PDFs that did not fit
(`quire sync hub now` says *2 PDF(s) not uploaded: the hub's storage is full*).

## Upgrades

Build or pull the newer image, then recreate the container; the volume, the hub's ID and everyone's
data stay:

```bash
podman build -f deploy/hub/Dockerfile -t localhost/quire-hub:local .   # or: podman compose pull
podman compose up -d --force-recreate
```

## Backups

The volume holds the database (`hub.sqlite`), the sealed files (`files/`) and the hub's key
(`hub.key`). **The key is the hub's identity**: devices dial the hub by it. Losing it means a new hub
ID, and every device adding the hub again; anyone who has it can pose as your hub (still without
reading anything: the data is sealed by your devices). Keep a copy apart, for example in your
password manager:

```bash
podman compose cp hub:/data/hub.key ./hub.key      # comes out readable only by you
```

**Back up the whole volume** with the hub stopped (the database is then consistent). The volume's
name is the folder's name plus `_quire-hub-data`; `podman volume ls` shows it.

```bash
podman compose stop
podman run --rm -v quire-hub_quire-hub-data:/data:ro -v "$PWD":/backup:Z \
  docker.io/library/busybox:1.37 tar czf /backup/hub-backup.tar.gz -C /data .
chmod 600 hub-backup.tar.gz        # it holds hub.key too
podman compose start
```

(`:Z` lets the container write the folder under SELinux; Docker without SELinux ignores it.)

**Restore** into a new, empty volume before starting the hub on it:

```bash
podman volume create quire-hub_quire-hub-data
podman run --rm -v quire-hub_quire-hub-data:/data -v "$PWD":/backup:ro,Z \
  docker.io/library/busybox:1.37 tar xzpf /backup/hub-backup.tar.gz -C /data
podman compose up -d
podman compose logs hub      # the same ID as before, and no setup code
```

Tested end to end: the restored hub had the same ID and accounts, and a device synced with it with
no new registration. A volume created by hand like this is left alone by `podman compose down -v`.

For a hub many people rely on, `deploy/cloud/` runs the same image with continuous database
backups to object storage (Litestream), the files copied there every ten minutes (rclone), and
your own relay; its README is the runbook, including a rehearsed restore.

## Troubleshooting

| You see | Means |
|---|---|
| *Cannot connect to the Docker daemon at unix:///run/user/…/podman.sock* | the Podman socket is off: `systemctl --user enable --now podman.socket` |
| *no relay reachable: only devices that reach this hub directly can sync* (hub log) | the hub has no internet route to a relay; devices on its network still work |
| *this code is wrong, used or expired* | a setup code or invitation used before, or an invitation older than 7 days: `quire-hub invite` makes a new one |
| *your storage on the hub is full* | the account's quota (`QUIRE_HUB_QUOTA_MB`); changes still sync, some PDFs wait |
| `quire sync hub add` waits, then fails | the device cannot reach the hub: check the hub is up (`podman ps`) and online (its log) |
