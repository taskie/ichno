CREATE TABLE IF NOT EXISTS `namespaces` (
    `id` VARCHAR(128) NOT NULL,
    `url` VARCHAR(512) NOT NULL,
    `type` INTEGER NOT NULL,
    `history_id` INTEGER,           -- FK,
    `version` INTEGER,              -- cached from history
    `status` INTEGER,               -- cached from history
    `mtime` DATETIME,               -- cached from history
    `object_id` INTEGER,            -- cached from history, FK
    `digest` CHAR(64),              -- cached from object
    `size` BIGINT,                  -- cached from object
    `fast_digest` BIGINT,           -- cached from object
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE (`digest`)
) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE INDEX `namespaces_url_id` ON `namespaces` (`url`, `id`);
CREATE INDEX `namespaces_status_id` ON `namespaces` (`status`, `id`);
CREATE INDEX `namespaces_updated_at_id` ON `namespaces` (`updated_at`, `id`);
