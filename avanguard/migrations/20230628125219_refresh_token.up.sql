CREATE TABLE  "refreshtoken" (
    id BIGSERIAL PRIMARY KEY,
    wallet_id BIGINT NOT NULL,
    token TEXT NOT NULL,
    expires_at BIGINT NOT NULL,
    used_at BIGINT NULL,
    blacklisted BOOLEAN NOT NULL
);
ALTER TABLE refreshtoken ADD FOREIGN KEY(wallet_id) REFERENCES "wallet"(id) ON DELETE CASCADE;
