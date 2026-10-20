-- When each part of a reference last changed (JSON: part name → milliseconds), so two devices'
-- edits of different fields both survive a sync (`quire_core::sync::merge_reference`).
ALTER TABLE reference ADD COLUMN clocks TEXT NOT NULL DEFAULT '{}';
