select idn,idp,position,label,email,content,event
from emails.notes
where idu=$1 -- 1|2
order by idp, position
;