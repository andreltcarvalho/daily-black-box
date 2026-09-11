PRAGMA foreign_keys = ON;
CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE categories (id TEXT PRIMARY KEY, name TEXT NOT NULL, position INTEGER NOT NULL, distraction INTEGER NOT NULL DEFAULT 0 CHECK(distraction IN (0,1)));
INSERT INTO categories(id,name,position) VALUES
 ('vdi','VDI / trabalho',0),('local','Trabalho local',1),('social','Redes sociais',2),
 ('video','Vídeo',3),('entertainment','Entretenimento',4),('research','Pesquisa',5),
 ('communication','Comunicação',6),('system','Sistema',7),('unknown','Desconhecido',8);
CREATE TABLE domain_categories (hostname TEXT PRIMARY KEY, category TEXT NOT NULL REFERENCES categories(id), updated_at INTEGER NOT NULL);
CREATE TABLE sessions (
 id INTEGER PRIMARY KEY, run_id TEXT NOT NULL, start_utc INTEGER NOT NULL, end_utc INTEGER NOT NULL,
 duration_ms INTEGER NOT NULL CHECK(duration_ms>=0), local_date TEXT NOT NULL, offset_seconds INTEGER NOT NULL,
 source TEXT NOT NULL CHECK(source IN ('vdi','browser','system','unknown','idle','paused','unobserved')),
 hostname TEXT, reason TEXT NOT NULL, is_open INTEGER NOT NULL DEFAULT 0 CHECK(is_open IN (0,1)),
 CHECK(hostname IS NULL OR source='browser')
);
CREATE UNIQUE INDEX only_one_open ON sessions(is_open) WHERE is_open=1;
CREATE INDEX sessions_day ON sessions(local_date,id);
CREATE INDEX sessions_domain ON sessions(hostname,local_date);
CREATE TABLE accesses (id INTEGER PRIMARY KEY, run_id TEXT NOT NULL, at_utc INTEGER NOT NULL, local_date TEXT NOT NULL, hostname TEXT NOT NULL, tab_key TEXT NOT NULL);
CREATE INDEX accesses_day ON accesses(local_date,hostname);
CREATE TABLE tracker_checkpoint (id INTEGER PRIMARY KEY CHECK(id=1), confirmed_utc INTEGER NOT NULL, clean_exit INTEGER NOT NULL);
PRAGMA user_version = 1;
