# quire hub, hosted (runbook)

The paid service is the same hub image as self-hosting (`deploy/docker-compose.yml`), run for many
accounts, with backups and our own iroh relay. Everything here is sealed by the users' devices: the
service holds no readable library data (`.knowledge/sync-hub.md`).

## What runs

| Service | Image | Does |
|---|---|---|
| `hub` | `ghcr.io/neuro-kitchen/quire-hub` | accounts, devices, sealed batches and files |
| `litestream` | `litestream/litestream:0.5.17` | the hub's database to object storage, continuously |
| `files-backup` | `rclone/rclone:1.75.1` | sealed files to the same storage, every 10 minutes |
| `relay` | `n0computer/iroh-relay:v1.3.0` | relay for devices that cannot connect directly |

One small server (2 vCPU, 4 GB, a disk sized for the files) runs it all; object storage holds the
backups (Cloudflare R2 has no download fees; Backblaze B2 and Hetzner Object Storage are cheap).

## Set up

1. A server with Docker; DNS `relay.<our domain>` → its address; firewall open for 80/tcp,
   443/tcp (the relay's certificates and HTTPS) and 7842/udp (the relay's QUIC). The hub itself
   needs no inbound port.
2. A bucket and an access key on the object storage.
3. `cp .env.example .env` and fill it; in `iroh-relay.toml` put the relay's hostname and our
   contact address.
4. `docker compose up -d`; `docker compose logs hub` shows the hub's ID (what customers' apps add).
5. **Back up `hub.key` now** (`docker compose cp hub:/data/hub.key .`), into the password manager or
   the provider's secret store, not the backup bucket: it is the hub's identity (devices dial it by
   it). Losing it means a new hub ID and every customer re-adding the hub; leaking it lets someone
   pose as the hub (they still read nothing: data is sealed by the devices).

## Accounts

The account page (to build: sign-up, passkey, payment, see `.knowledge/sync-hub.md`, "still the
owner's") makes an account and shows its link code:

```bash
docker compose exec hub quire-hub account new --quota-mb 10240   # account <id>, link code XXXX-XXXX-XXXX
```

The customer enters it once in the app (`quire sync hub add <hub-id> <link-code>`). Deleting an
account (their request, or the end of their subscription after the notice period) removes its
batches, snapshots, files and devices:

```bash
docker compose exec hub quire-hub account delete <id>
```

## Restore

Rehearsed on 2026-10-21 (`rclone serve s3` standing in for the object storage): the volume
destroyed, then restored as below; the hub came back with the same ID and its accounts, the
existing device synced, and a new device joined and downloaded a PDF byte-identical.

```bash
docker compose stop hub litestream files-backup
docker volume rm <project>_hub-data            # only if it is damaged; a new one is made empty
docker compose run --rm --user 0 litestream restore -config /etc/litestream.yml /data/hub.sqlite
docker compose run --rm restore copy backup:<bucket>/files /data/files
# hub.key from the password manager into the volume, then everything to the hub's user (65532):
docker compose run --rm -v "$PWD/hub.key:/hub.key:ro" --entrypoint /bin/sh restore \
  -c 'cp /hub.key /data/hub.key && chmod 600 /data/hub.key && chown -R 65532:65532 /data'
docker compose up -d
docker compose logs hub     # the same hub ID, and no setup code (the accounts are back)
```

## Watch

- `docker compose ps`: the hub's health check (`quire-hub health`).
- An external uptime check on the relay's HTTPS (`https://relay.<our domain>`).
- Logs carry counts only (sessions, batches, files), never content.

## Not done yet

- The account page and payments (merchant of record recommended), pricing, terms of service,
  privacy policy, abuse policy: the owner's decisions.
- Relay access limited to registered devices (`access.http` calling the hub); for now anyone may
  relay through it (end-to-end encrypted, rate limits to add).
- A restore drill on a real second server with the real object storage (rehearsed locally).
