use std::fs;

use lettre::{Message, SendmailTransport, Transport};
use lettre::message::{header, MultiPart, SinglePart};
use lol_html::{comments, element, HtmlRewriter, Settings};
use lol_html::html_content::ContentType;

use shared::types::BoxMailAttachments;

use crate::constants::path_to_temp_with_ind;

pub async fn send_message(sender: &str, recipient: &str, subject: &str, message: &str, attachments: &Option<BoxMailAttachments>) -> bool {
    let mut result = false;

    let text = to_text(message);

    let mut multipart = MultiPart::alternative()
        .singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_PLAIN)
                .body(text),
        )
        .singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(message.to_string()),
        );

    if let Some(attachments) = attachments {
        if attachments.list.len() > 0 {
            let key = attachments.key.clone();
            multipart = MultiPart::mixed().multipart(multipart);
            for item in attachments.list.iter() {
                let path_to_file = path_to_temp_with_ind(&key, &item.id);
                let mime = mime_guess::from_path(&item.file_name).first_or_octet_stream();
                if let Ok(content_type) = header::ContentType::parse(&mime.to_string()) {
                    if let Ok(f) = fs::read(path_to_file) {
                        multipart = multipart.singlepart(
                            SinglePart::builder()
                                .header(content_type)
                                .header(header::ContentDisposition::attachment(&item.file_name))
                                .header(header::ContentTransferEncoding::Base64)
                                .body(f)
                        );
                    }
                }
            }
        }
    }

    if let Ok(sender) = sender.parse() {
        if let Ok(recipient) = recipient.parse() {
            let message = Message::builder()
                .from(sender)
                .to(recipient)
                .subject(subject)
                .multipart(multipart);

            match message {
                Ok(email) => {
                    let mailer = SendmailTransport::new();
                    match mailer.send(&email) {
                        Ok(_) => {
                            result = true;
                        }
                        Err(err) => {
                            tracing::error!("Could not send email: {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!("{:?}", err);
                }
            }
        }
    }

    result
}

fn to_text(html: &str) -> String {
    let mut output = vec![];

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("a", |el| {
                    el.remove_and_keep_content();
                    if let Some(href) = el.get_attribute("href") {
                        el.after(&format!(" [{href}]"), ContentType::Text);
                    }
                    Ok(())
                }),
                element!("li", |el| {
                    el.before("* ", ContentType::Text);
                    el.after("\n", ContentType::Text);
                    el.remove_and_keep_content();
                    Ok(())
                }),
                element!("script,style,meta,base,link,title", |el| {
                    el.remove();
                    Ok(())
                }),
                element!("p,h1,h2,h3,h4,h5,h6,div,blockquote,pre,br", |el| {
                    el.after("\n", ContentType::Text);
                    el.remove_and_keep_content();
                    Ok(())
                }),
                element!("*", |el| {
                    el.remove_and_keep_content();
                    Ok(())
                }),
                comments!("*", |c| {
                    c.remove();
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    if let Err(err) = rewriter.write(html.as_bytes()) {
        tracing::error!("to_text[1] {:?}", err);
    }
    if let Err(err) = rewriter.end() {
        tracing::error!("to_text[2] {:?}", err);
    }

    let text = String::from_utf8(output).unwrap_or_default();

    text.replace("&gt;", ">")
        .split("\n")
        .into_iter()
        .filter_map(|t| if t.trim().is_empty() { None } else { Some(t) })
        .collect::<Vec<_>>()
        .join("\n")
}