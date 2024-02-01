CREATE TABLE defeat_record
(
    "user"   TEXT    NOT NULL,
    password TEXT    NOT NULL,
    count    INTEGER NOT NULL,
    CONSTRAINT defeat_record_pk PRIMARY KEY ("user", password)
);