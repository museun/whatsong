BEGIN;

CREATE TABLE IF NOT EXISTS `youtube_videos` (
	`id`		INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT UNIQUE,
	`vid`	    TEXT NOT NULL,
	`ts`    	INTEGER NOT NULL,
	`duration`	INTEGER NOT NULL,
	`title`	    TEXT NOT NULL
);

COMMIT;
