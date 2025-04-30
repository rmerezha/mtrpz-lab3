BEGIN;

CREATE TABLE users
(
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(100)       NOT NULL,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE notes
(
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    title      VARCHAR(255) NOT NULL,
    content    TEXT         NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notes_search ON notes USING GIN (to_tsvector('english', title || ' ' || content));
CREATE INDEX idx_notes_user_id ON notes(user_id);

CREATE TABLE shared_notes
(
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    note_id     UUID NOT NULL REFERENCES notes (id) ON DELETE CASCADE,
    shared_with UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    shared_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_shared_notes_unique ON shared_notes (note_id, shared_with);
CREATE INDEX idx_shared_notes_shared_with ON shared_notes (shared_with);

CREATE TABLE user_token
(
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ                NOT NULL DEFAULT now(),
    user_id    UUID NOT NULL REFERENCES users (id)  ON DELETE CASCADE,
    token_type VARCHAR(16)                NOT NULL,
    token      VARCHAR(64)                NOT NULL,
    status     VARCHAR(16)                NOT NULL DEFAULT 'ACTIVE',
    expires_at TIMESTAMPTZ                NOT NULL
);

CREATE UNIQUE INDEX idx_user_token_token_token_type ON user_token (token, token_type);
CREATE INDEX idx_user_token_user_id_expires_at ON user_token (user_id, expires_at);


CREATE TABLE db_version
(
    name    VARCHAR(64) PRIMARY KEY,
    version INT NOT NULL
);

INSERT INTO db_version(name, version)
VALUES ('speer-test', 1);

COMMIT;
