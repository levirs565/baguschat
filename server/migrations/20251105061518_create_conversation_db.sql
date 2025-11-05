-- Add migration script here
CREATE TYPE conversation_type AS ENUM ('text', 'file', 'image');
CREATE TABLE conversations(
    id UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW() ,
    sender_id UUID NOT NULL REFERENCES users(id),
    receiver_id UUID REFERENCES users(id),
    sender_key BYTEA NOT NULL,
    receiver_key BYTEA NOT NULL,
    contet_type conversation_type NOT NULL
);
CREATE TABLE conversations_text(
    id UUID PRIMARY KEY NOT NULL,
    cipher BYTEA NOT NULL
);
CREATE TABLE conversations_image(
    id UUID PRIMARY KEY NOT NULL,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size BIGINT NOT NULL,
    path TEXT NOT NULL
);