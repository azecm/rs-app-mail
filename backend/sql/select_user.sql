select name, signature, email
from emails.users
where idu=$1 -- 1|2
;