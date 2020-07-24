CREATE TABLE IF NOT EXISTS `contents` (
    `object_id` INTEGER NOT NULL,  -- FK
    `digest` CHAR(64) NOT NULL,    -- cached from object
    `body` BLOB NOT NULL,
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`object_id`),
    UNIQUE (`digest`)
) CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE INDEX `contents_updated_at` ON `contents` (`updated_at`, `object_id`);
