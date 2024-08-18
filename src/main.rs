mod providers;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use polib::catalog::Catalog;
use polib::message::MessageMutView;
use polib::message::MessageView;
use polib::po_file;
use std::env;
use std::path::Path;

use crate::providers::moonshot_translate;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <po_file>", args[0]);
        std::process::exit(1);
    }

    let po_file = &args[1];

    let mut catalog = load_catalog(po_file)?;

    translate(&mut catalog, 1000).await?;

    save_catalog(po_file, &catalog)?;

    println!("Translation completed.");

    Ok(())
}

async fn translate(catalog: &mut Catalog, max_char_count: usize) -> Result<()> {
    let mut char_count = 0;
    let mut msg_indices = Vec::new();
    let mut msgids = Vec::new();

    for (idx, msg) in catalog.messages().enumerate() {
        if msg.is_translated() {
            continue;
        }
        let msgid = msg.msgid();
        if msgid.trim().is_empty() || msgid.contains("```") || msgid.contains(';') {
            continue;
        }
        char_count += msgid.chars().count();
        if char_count > max_char_count {
            let trunc = msgid.chars().take(20).collect::<String>();
            eprintln!("Stopping translation at message {idx}: {trunc}");
            break;
        }

        msg_indices.push(idx);
        msgids.push(msgid);
    }

    eprintln!(
        "Translating {} messages into {}",
        msg_indices.len(),
        catalog.metadata.language
    );
    let translations = moonshot_translate(&msgids, &catalog.metadata.language).await?;

    for mut message in catalog.messages_mut() {
        if !message.is_translated() && message.is_singular() {
            if let Some(translated) = translations.get(message.msgid()) {
                message.set_msgstr(translated.to_string())?;
            }
        }
    }

    Ok(())
}

fn load_catalog<P: AsRef<Path>>(path: P) -> Result<Catalog> {
    po_file::parse(path.as_ref())
        .map_err(|err| anyhow!("{err}"))
        .with_context(|| format!("Could not parse {} as PO file", path.as_ref().display()))
}

fn save_catalog<P: AsRef<Path>>(path: P, catalog: &Catalog) -> Result<()> {
    let path = path.as_ref();
    if path.exists() {
        // po_file::write does not remove an existing file
        std::fs::remove_file(path).with_context(|| format!("Removing {}", path.display()))?
    }
    polib::po_file::write(catalog, path)
        .with_context(|| format!("Writing catalog to {}", path.display()))?;
    Ok(())
}
