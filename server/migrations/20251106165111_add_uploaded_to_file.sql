-- Add migration script here
ALTER TABLE conversations_image
    RENAME TO conversations_file;
ALTER TABLE conversations_file
    ADD COLUMN uploaded BOOLEAN NOT NULL DEFAULT FALSE;