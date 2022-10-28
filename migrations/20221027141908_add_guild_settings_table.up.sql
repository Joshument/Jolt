-- Add up migration script here
CREATE TABLE guild_settings (
    guild_id BIGINT NOT NULL UNIQUE,
    mute_role_id BIGINT,
    logs_channel_id BIGINT,
    prefix TEXT
)