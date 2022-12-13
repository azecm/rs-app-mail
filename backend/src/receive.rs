use std::fs;
use std::fs::File;
use std::io::Read;
use std::iter::Iterator;
use std::string::ToString;

use lol_html::{comments, element, HtmlRewriter, Settings};
use lol_html::html_content::ContentType;
use mail_parser::{Addr, HeaderValue, Message, MimeHeaders};
use mail_parser::PartType::Binary;
use once_cell::sync::Lazy;
use uuid::Uuid;

use crate::constants::{MAIL_SOURCE_PATH, path_to_attachment, path_to_saved};
use crate::db_boxes::db_box_add_received;
use crate::db_types::{DBMailAddress, DBMailAttachmentItem, DBMailAttachments};
use crate::state::USER_BY_EMAIL;
use crate::utils::{get_dir_path, get_file_name};

static TRUSTED: Lazy<Vec<String>> = Lazy::new(|| [
    ".ru", ".dev", ".org", ".net", "gmail.com", "hotmail.com", "zoom.us"
].iter().map(|v| v.to_string()).collect::<Vec<String>>());

// https://crates.io/crates/notify

pub async fn mail_watcher() {
    let mut interval_timer = tokio::time::interval(chrono::Duration::seconds(5).to_std().unwrap());
    loop {
        interval_timer.tick().await;
        for path_dir in watch_dirs() {
            match fs::metadata(&path_dir) {
                Ok(_) => {
                    match fs::read_dir(&path_dir) {
                        Ok(read_dir) => {
                            for entries in read_dir.into_iter() {
                                if let Ok(entries) = entries {
                                    if let Some(path_to_file) = entries.path().to_str() {
                                        read_email(path_to_file);
                                    }
                                }
                            }
                        }
                        Err(err) => tracing::error!("watch_dirs[1]: {path_dir} -- {err}")
                    }
                }
                Err(err) => tracing::error!("watch_dirs[2]: {path_dir} -- {err}")
            }
        }
    }
}

/*
pub fn mail_watcher_() -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;

    for path_dir in watch_dirs() {
        if let Ok(_) = fs::metadata(&path_dir) {
            if let Err(err) = watcher.watch(Path::new(&path_dir), RecursiveMode::NonRecursive) {
                tracing::error!("mail_watcher: {:?}", err);
            }
        }
    }

    tokio::task::spawn_blocking(move || {
        loop {
            if let Ok(Ok(event)) = rx.recv() {
                if event.kind ==  Create(CreateKind::File) {
                    for item in event.paths.iter() {
                        if let Some(path_to_file) = item.as_path().to_str() {
                            read_email(path_to_file);
                        }
                    }
                }
            }
        }
    });

    Ok(())
}
*/

fn watch_dirs() -> Vec<String> {
    let emails = if let Ok(users) = USER_BY_EMAIL.lock() {
        users.keys().map(|k| k.clone()).collect::<Vec<String>>()
    } else { vec![] };

    emails.iter().map(|email| {
        let list = email.split("@").collect::<Vec<_>>();
        let domain = list[1];
        let user = list[0];
        format!("{MAIL_SOURCE_PATH}/{domain}/{user}/new")
    }).collect::<Vec<_>>()
}

fn get_email_from_path(path: &str) -> String {
    let list = path.split("/").collect::<Vec<_>>();
    format!("{}@{}", list[list.len() - 3], list[list.len() - 4])
}

fn read_email(path_to_file: &str) {
    if get_file_name(path_to_file).starts_with(".") {
        return;
    }
    match fs::metadata(path_to_file) {
        Ok(_) => {}
        Err(_) => {
            return;
        }
    }
    let current_email = get_email_from_path(path_to_file);
    let mail_source: Vec<u8> = if let Ok(mut f) = File::open(path_to_file) {
        let mut d: Vec<u8> = Vec::<u8>::new();
        if let Err(_) = f.read_to_end(&mut d) {}
        d
    } else { vec![] };

    match Message::parse(&mail_source) {
        Some(message) => prepare(message, &current_email),
        None => tracing::error!("parse_mail error")
    }
    /*match mailparse::parse_mail(&mail_source) {
        Ok(mail) => prepare(mail, &current_email),
        Err(err) => tracing::error!("parse_mail: {:?}", err)
    }*/

    let target_file_path = path_to_saved(&current_email, &get_file_name(path_to_file));
    match fs::create_dir_all(get_dir_path(&target_file_path)) {
        Ok(_) => {
            if let Err(err) = fs::rename(path_to_file, &target_file_path) {
                tracing::error!("read_email[1]: {:?}", err);
            }
        }
        Err(err) => {
            tracing::error!("read_email[2]: {:?}", err);
        }
    }
}

fn prepare(message: Message, current_email: &str) {
    let from = message.get_from();
    let to = message.get_to();
    let subject = message.get_subject().unwrap_or_default().to_string();
    let html = match message.get_html_body(0) {
        Some(html) => html.to_string(),
        None => "".to_string()
    };
    let text = match message.get_text_body(0) {
        Some(text) => text.to_string(),
        None => "".to_string()
    };

    let content = if !html.is_empty() {
        match clean_html(&html) {
            Ok(html) => html,
            Err(err) => {
                tracing::error!("clean_html: {:?}", err);
                "".to_string()
            }
        }
    } else if !text.is_empty() {
        format!("<pre>{}</pre>", text)
    } else {
        "".to_string()
    };

    let key = Uuid::new_v4().to_string();
    let mut list: Vec<DBMailAttachmentItem> = vec![];
    for part in message.get_attachments() {
        if part.is_binary() {
            if let Some(file_name) = part.get_attachment_name() {
                if let Binary(body) = &part.body {
                    let id = list.len() + 1;
                    let size = body.len() as u64;

                    let file_path = path_to_attachment(current_email, &key, &id);
                    match fs::create_dir_all(get_dir_path(&file_path)) {
                        Ok(_) => {
                            match fs::write(&file_path, body) {
                                Ok(_) => {
                                    list.push(DBMailAttachmentItem { id, size, file_name: file_name.to_string() });
                                }
                                Err(err) => {
                                    tracing::error!("save attachment: {:?}", err);
                                }
                            }
                        }
                        Err(err) => {
                            tracing::error!("save attachment: {:?}", err);
                        }
                    }
                }
            }
        }
    }

    let attachments: Option<DBMailAttachments> = if list.len() == 0 { None } else {
        Some(DBMailAttachments { key: key.clone(), list })
    };

    let sender = mail_address_from_header(from);
    let recipient = mail_address_from_header(to);

    //let from_text = format!("{:?}", from).to_lowercase();
    let flag_spam = !format!("{:?}", to).to_lowercase().contains(current_email);

    let mut flag_ends_trusted = false;
    for ends in TRUSTED.iter() {
        if sender.address.ends_with(ends) {
            flag_ends_trusted = true;
            break;
        }
    }
    tracing::info!("{:?} {:?}", sender, flag_ends_trusted);

    db_box_add_received(
        flag_spam,
        current_email.to_string(),
        sender,
        recipient,
        subject,
        content,
        attachments,
    );
}

fn mail_address_from_header(header: &HeaderValue) -> DBMailAddress {
    match header {
        HeaderValue::Address(addr) => {
            get_first(addr)
        }
        HeaderValue::AddressList(addr_list) => {
            if addr_list.len() > 0 {
                get_first(&addr_list[0])
            } else {
                DBMailAddress::default()
            }
        }
        _ => {
            tracing::warn!("mail_address_from_header: {:?}", header);
            DBMailAddress::default()
        }
    }
}

fn get_first(addr: &Addr) -> DBMailAddress {
    let name = match &addr.name {
        Some(val) => val.to_string(),
        None => "".to_string()
    };
    let address = match &addr.address {
        Some(val) => val.to_string(),
        None => "".to_string()
    };
    DBMailAddress { address, name: if name.is_empty() { None } else { Some(name) } }
}

pub fn get_email(text: &str) -> (Option<String>, String) {
    if let Ok(data) = mailparse::addrparse(text) {
        if let Some(email) = data.first() {
            match email {
                mailparse::MailAddr::Group(info) => {
                    if let Some(info) = info.addrs.first() {
                        return (info.display_name.clone(), info.addr.clone());
                    }
                }
                mailparse::MailAddr::Single(info) => {
                    return (info.display_name.clone(), info.addr.clone());
                }
            }
        }
    }
    (None, "".to_string())
}

fn clean_html(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = vec![];

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("script,style,meta,base,link,title", |el| {
                    el.remove();
                    Ok(())
                }),
                element!("img", |el| {
                    if let Some(alt) = el.get_attribute("alt") {
                        if alt.trim().is_empty() {
                            el.remove();
                        }
                        else {
                            el.replace(&format!("[{alt}]"), ContentType::Text);
                        }
                    }
                    else {
                        el.remove();
                    }
                    Ok(())
                }),
                element!("[style]", |el| {
                    el.remove_attribute("style");
                    Ok(())
                }),
                element!("[bgcolor]", |el| {
                    el.remove_attribute("bgcolor");
                    Ok(())
                }),
                element!("[background]", |el| {
                    el.remove_attribute("background");
                    Ok(())
                }),
                element!("[bordercolor]", |el| {
                    el.remove_attribute("bordercolor");
                    Ok(())
                }),
                element!("[class]", |el| {
                    el.remove_attribute("class");
                    Ok(())
                }),
                element!("[color]", |el| {
                    el.remove_attribute("color");
                    Ok(())
                }),
                comments!("*", |c| {
                    c.remove();
                    Ok(())
                }),
                element!("a", |el| {
                    el.prepend("*", ContentType::Text);
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(source.as_bytes())?;
    rewriter.end()?;

    let html = String::from_utf8(output)?;
    Ok(html)
}