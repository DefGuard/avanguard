CREATE TABLE  "refreshtoken" (
    id BIGSERIAL PRIMARY KEY,
    wallet_id BIGINT NOT NULL,
    token TEXT NOT NULL,
    expires_at timestamp without time zone NOT NULL,
    used_at timestamp without time zone NULL,
    blacklisted_at timestamp without time zone,
    FOREIGN KEY(wallet_id) REFERENCES "wallet"(id)
);
