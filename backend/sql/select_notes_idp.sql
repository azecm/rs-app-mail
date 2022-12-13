select COUNT(*)
from emails.notes
where idu=$1 and idp=$2
--where idu=1 and idp=1
;
