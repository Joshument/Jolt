-- Add up migration script here
CREATE TABLE timed_moderations (
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    moderation_type TINYINT NOT NULL,
    expiry_date BIGINT NOT NULL,
    reason TEXT
)