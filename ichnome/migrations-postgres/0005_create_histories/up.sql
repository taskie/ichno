CREATE TABLE IF NOT EXISTS "histories" (
    "id" integer NOT NULL PRIMARY KEY GENERATED BY DEFAULT AS IDENTITY,
    "workspace_id" integer NOT NULL,  -- cached from group, FK
    "group_id" integer NOT NULL,      -- FK
    "path" varchar(512) NOT NULL,
    "version" integer NOT NULL,
    "status" integer NOT NULL,
    "mtime" timestamptz,
    "footprint_id" integer,           -- FK
    "digest" char(64),                -- cached from footprint
    "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE ("group_id", "path", "version")
)
;

ALTER TABLE "histories" ADD CONSTRAINT "fk_histories_workspace_id_group_id" FOREIGN KEY ("workspace_id", "group_id") REFERENCES "groups" ("workspace_id", "id");
ALTER TABLE "histories" ADD CONSTRAINT "fk_histories_footprint_id" FOREIGN KEY ("footprint_id") REFERENCES "footprints" ("id");

CREATE INDEX "ix_histories_workspace_id_updated_at" ON "histories" ("workspace_id", "updated_at");
CREATE INDEX "ix_histories_workspace_id_footprint_id_path_version" ON "histories" ("workspace_id", "footprint_id", "path", "version");
CREATE INDEX "ix_histories_workspace_id_footprint_id_mtime" ON "histories" ("workspace_id", "footprint_id", "mtime");
