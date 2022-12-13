use crate::types::MailBoxes;

pub fn box_type_index(mb: &MailBoxes) -> usize {
    match *mb {
        MailBoxes::Inbox => 0,
        MailBoxes::Ready => 1,
        MailBoxes::Sent => 2,
        MailBoxes::Trash => 3,
        MailBoxes::Notes => 4,
    }
}