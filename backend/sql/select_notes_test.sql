--select idp from emails.notes where idu=1 and idn=1;
-- `$$${val}$$`

--select event from emails.notes where idn = 24;
--update emails.notes set event='{"date": "2022-10-20", "delta": 1}'::jsonb where idn = 24;
--update emails.notes set event=$${"date": "2022-10-21", "delta": 1}$$::jsonb where idn = 24;
--update emails.notes set event=NULL where idn = 24;
--select event from emails.notes where idn = 24;

select attachments from emails.boxes where idb=29668;