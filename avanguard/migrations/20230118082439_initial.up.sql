CREATE TABLE "wallet" (
    id bigserial PRIMARY KEY,
    address text NOT NULL UNIQUE,
    challenge_message text NOT NULL,
    challenge_signature text NULL,
    creation_timestamp timestamp without time zone NOT NULL,
    validation_timestamp timestamp without time zone NULL
);
