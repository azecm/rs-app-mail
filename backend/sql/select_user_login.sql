select idu
from emails.users
where email=$1 and name=$2 and password=$3
;