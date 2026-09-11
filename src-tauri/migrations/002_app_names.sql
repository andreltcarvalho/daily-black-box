ALTER TABLE sessions RENAME TO sessions_v1;

CREATE TABLE sessions (
 id INTEGER PRIMARY KEY, run_id TEXT NOT NULL, start_utc INTEGER NOT NULL, end_utc INTEGER NOT NULL,
 duration_ms INTEGER NOT NULL CHECK(duration_ms>=0), local_date TEXT NOT NULL, offset_seconds INTEGER NOT NULL,
 source TEXT NOT NULL CHECK(source IN ('vdi','browser','app','system','unknown','idle','paused','unobserved')),
 hostname TEXT, app_name TEXT, reason TEXT NOT NULL, is_open INTEGER NOT NULL DEFAULT 0 CHECK(is_open IN (0,1)),
 CHECK(hostname IS NULL OR source='browser'),
 CHECK(app_name IS NULL OR source IN ('browser','app','system'))
);

INSERT INTO sessions(id,run_id,start_utc,end_utc,duration_ms,local_date,offset_seconds,source,hostname,app_name,reason,is_open)
SELECT id,run_id,start_utc,end_utc,duration_ms,local_date,offset_seconds,source,hostname,NULL,reason,is_open
FROM sessions_v1;

DROP TABLE sessions_v1;
CREATE UNIQUE INDEX only_one_open ON sessions(is_open) WHERE is_open=1;
CREATE INDEX sessions_day ON sessions(local_date,id);
CREATE INDEX sessions_domain ON sessions(hostname,local_date);
CREATE INDEX sessions_app ON sessions(app_name,local_date);
PRAGMA user_version = 2;
