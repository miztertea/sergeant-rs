use mail_parser::{MessageParser, MimeHeaders};
use std::fs;

fn main() {
    let names = [
        "01-plain-text.eml",
        "02-multipart-alternative.eml",
        "03-with-attachment.eml",
        "04-nested-rfc822.eml",
        "05-encoding-zoo.eml",
        "06-malformed-no-headers.eml",
        "07-broken-mime-diagnostic-only.eml",
    ];
    let parser = MessageParser::default();
    for name in names {
        println!("=== {name} ===");
        let bytes = fs::read(name).expect("read fixture");
        match parser.parse(&bytes) {
            None => println!("  parse() -> None (no headers found)"),
            Some(msg) => {
                println!("  parse() -> Some(Message)");
                println!("  header count: {}", msg.headers().len());
                println!("  subject: {:?}", msg.subject());
                println!(
                    "  from: {:?}",
                    msg.from().map(|a| format!("{a:?}"))
                );
                println!("  body_text(0): {:?}", msg.body_text(0));
                println!("  body_html(0): {:?}", msg.body_html(0));
                println!("  text_body count: {}", msg.text_body.len());
                println!("  html_body count: {}", msg.html_body.len());
                println!("  attachment count: {}", msg.attachments().count());
                for (i, att) in msg.attachments().enumerate() {
                    println!(
                        "    attachment[{i}] name={:?} len={}",
                        att.attachment_name(),
                        att.contents().len()
                    );
                    if let Some(nested) = att.message() {
                        println!(
                            "      nested message subject={:?} body_text(0)={:?}",
                            nested.subject(),
                            nested.body_text(0)
                        );
                    }
                }
            }
        }
        println!();
    }
}

#[allow(dead_code)]
fn probe_broken_mime_diagnostic_only() {}
