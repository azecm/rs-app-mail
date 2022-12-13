select idb, date, unread, sender, recipient, subject, content, attachments
from emails.boxes
where idu=$1 and box=$2
order by date desc
offset $3
limit $4
;