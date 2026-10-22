-- What happened on this device, for its notifications pane (`.tasks/T14-security.md`): what syncs
-- brought from which device, and security events. This device only: no change-log triggers,
-- never synced, never sent to a hub.
CREATE TABLE activity (
    id      INTEGER PRIMARY KEY,
    at      INTEGER NOT NULL,             -- milliseconds since 1970
    kind    TEXT NOT NULL,                -- sync, linked, link_allowed, link_refused, wrong_code,
                                          -- unlinked, key_changed, new_device
    source  TEXT,                         -- the device it concerns, by name (as it named itself)
    detail  TEXT NOT NULL DEFAULT '',     -- sync: `note=2,highlight=1`; others: a number, or empty
    read    INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE INDEX activity_at ON activity (at);

-- This person's devices seen on the hub: a device not in here is new (noted once).
CREATE TABLE known_device (
    device      TEXT PRIMARY KEY,         -- its network ID
    first_seen  INTEGER NOT NULL          -- milliseconds since 1970
) STRICT;
