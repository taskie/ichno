CREATE TABLE IF NOT EXISTS `attributes` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `entity_type` INTEGER NOT NULL,
    `entity_id` INTEGER NOT NULL,        -- FK
    `namespace_id` VARCHAR(128),         -- cached from stat or history
    `path` VARCHAR(512),                 -- cached from stat or history
    `version` INTEGER,                   -- cached from history
    `digest` CHAR(64),                   -- cached from object
    `key` VARCHAR(128) NOT NULL,
    `value_object_id` INTEGER NOT NULL,  -- FK
    `value_digest` CHAR(64) NOT NULL,    -- cached from object
    `value_content_type` INTEGER NOT NULL,
    `status` INTEGER NOT NULL,
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE (`entity_type`, `entity_id`, `key`, `value_object_id`)
) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE INDEX `attributes_updated_at` ON `attributes` (`updated_at`, `id`);
CREATE INDEX `attributes_value_object_id_entity_type_entity_id_key` ON `attributes` (`value_object_id`, `entity_type`, `entity_id`, `key`);
