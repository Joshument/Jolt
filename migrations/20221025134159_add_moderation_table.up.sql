-- Add up migration script here
CREATE TABLE moderations (
    id BIGINT PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    moderation_type TINYINT NOT NULL,
    expiry_date BIGINT,
    reason TEXT,
    active BOOLEAN NOT NULL
)