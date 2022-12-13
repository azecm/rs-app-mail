
-- psql -U former -d sites -a -f update.sql

-- emails.notes
ALTER TABLE emails.notes ALTER COLUMN event DROP DEFAULT;
ALTER TABLE emails.notes ALTER COLUMN event DROP not null;
update emails.notes set event=null where event='{}'::jsonb;
--
-- emails.boxes
alter table emails.boxes alter column attachments drop not null;
alter table emails.boxes alter column attachments drop default;
update emails.boxes set attachments=null where attachments='{}'::jsonb;
--
