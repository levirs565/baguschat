CREATE TABLE users(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT UNIQUE NOT NULL,
    srp_salt BYTEA NOT NULL,
    srp_verifier BYTEA NOT NULL,
    public_key BYTEA NOT NULL,
    private_key_encrypted BYTEA NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
)