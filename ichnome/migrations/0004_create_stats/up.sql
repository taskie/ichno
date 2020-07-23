CREATE TABLE IF NOT EXISTS `stats` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `namespace_id` VARCHAR(128) NOT NULL,  -- FK
    `path` VARCHAR(512) NOT NULL,
    `history_id` INTEGER NOT NULL,         -- FK
    `status` INTEGER NOT NULL,             -- cached from history
    `mtime` DATETIME,                      -- cached from history
    `object_id` INTEGER,                   -- cached from history, FK
    `digest` CHAR(64),                     -- cached from object
    `size` BIGINT,                         -- cached from object
    `fast_digest` BIGINT,                  -- cached from object
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE (`namespace_id`, `path`)
) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE INDEX `stats_status_path` ON `stats` (`status`, `path`);
CREATE INDEX `stats_updated_at_path` ON `stats` (`updated_at`, `path`);
