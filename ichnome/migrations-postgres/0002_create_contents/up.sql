CREATE TABLE IF NOT EXISTS "contents" (
    "id" integer NOT NULL PRIMARY KEY GENERATED BY DEFAULT AS IDENTITY,
    "footprint_id" integer NOT NULL,  -- FK
    "body" bytea NOT NULL,
    "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE ("footprint_id")
)
;

ALTER TABLE "contents" ADD CONSTRAINT "fk_contents_footprint_id" FOREIGN KEY ("footprint_id") REFERENCES "footprints" ("id");

CREATE INDEX "ix_contents_created_at" ON "contents" ("created_at");
