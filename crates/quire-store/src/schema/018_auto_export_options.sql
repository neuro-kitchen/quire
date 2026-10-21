-- quire library schema, version 18: an automatic export can keep the options it was made with
-- (the app's export dialog), instead of naming a profile in the config.

ALTER TABLE auto_export ADD COLUMN options TEXT;  -- the options as JSON; NULL: the profile's
