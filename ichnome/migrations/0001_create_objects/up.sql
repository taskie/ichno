CREATE TABLE IF NOT EXISTS `objects` (
    `id` INTEGER NOT NULL AUTO_INCREMENT,
    `digest` CHAR(64) NOT NULL,
    `size` BIGINT NOT NULL,
    `fast_digest` BIGINT NOT NULL,
    `git_object_id` CHAR(40) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE (`digest`)
) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE INDEX `objects_git_object_id` ON `objects` (`git_object_id`);
