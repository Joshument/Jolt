-- Add up migration script here
CREATE TABLE moderations (
    id INTEGER NOT NULL,
    guild_id INTEGER NOT NULL,
    user_id BIGINT NOT NULL,
    moderator_id BIGINT NOT NULL,
    moderation_type TINYINT NOT NULL,
    administered_at BIGINT NOT NULL,
    expiry_date BIGINT,
    reason TEXT,
    active BOOLEAN NOT NULL,

    PRIMARY KEY (id, guild_id)
)
