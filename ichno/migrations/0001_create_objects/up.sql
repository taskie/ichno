CREATE TABLE IF NOT EXISTS `objects` (
    `id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `digest` CHAR(64) NOT NULL,
    `size` BIGINT NOT NULL,
    `fast_digest` BIGINT NOT NULL,
    `git_object_id` CHAR(40) NOT NULL,
    UNIQUE (`digest`)
);

CREATE INDEX `objects_git_object_id` ON `objects` (`git_object_id`);
