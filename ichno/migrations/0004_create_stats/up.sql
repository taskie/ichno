CREATE TABLE IF NOT EXISTS `stats` (
    `id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `namespace_id` VARCHAR(128) NOT NULL,  -- FK
    `path` VARCHAR(512) NOT NULL,
    `history_id` INTEGER NOT NULL,         -- FK
    `version` INTEGER NOT NULL,            -- cached from history
    `status` INTEGER NOT NULL,             -- cached from history
    `mtime` DATETIME,                      -- cached from history
    `object_id` INTEGER,                   -- cached from history, FK
    `digest` CHAR(64),                     -- cached from object
    `size` BIGINT,                         -- cached from object
    `fast_digest` BIGINT,                  -- cached from object
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (`namespace_id`, `path`)
);

CREATE INDEX `stats_status_path` ON `stats` (`status`, `path`);
CREATE INDEX `stats_updated_at_path` ON `stats` (`updated_at`, `path`);
