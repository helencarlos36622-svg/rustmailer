// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use std::{fs, io, time::Instant};

use native_db::Builder;
use tempfile::NamedTempFile;

use crate::modules::{cache::imap::minimal::MinimalEnvelope, database::ModelsAdapter};

fn create_temp_file() -> io::Result<NamedTempFile> {
    let temp_file = NamedTempFile::new()?;
    Ok(temp_file)
}

pub fn generate_envelopes(
    accounts: u64,
    mailboxes_per_account: u64,
    uids_per_mailbox: u32,
) -> Vec<MinimalEnvelope> {
    let mut envelopes =
        Vec::with_capacity((accounts * mailboxes_per_account * uids_per_mailbox as u64) as usize);

    let mut counter: u64 = 1;

    for account_id in 1..=accounts {
        for mailbox_id in 1..=mailboxes_per_account {
            for uid in 1..=uids_per_mailbox {
                envelopes.push(MinimalEnvelope {
                    account_id,
                    mailbox_id,
                    uid,
                    flags_hash: counter,
                });
                counter += 1;
            }
        }
    }

    envelopes
}

#[tokio::test]
async fn test1() {
    let mut adapter = ModelsAdapter::new();
    adapter.register_model::<MinimalEnvelope>();
    let models = adapter.models;
    let temp_file = create_temp_file().unwrap();
    let temp_path = temp_file.path().to_str().unwrap();

    // let database = Builder::new()
    //     .set_cache_size(1073741824)
    //     .create(&models, temp_path)
    //     .unwrap();
    let database = Builder::new().create_in_memory(&models).unwrap();

    let envelopes = generate_envelopes(20, 50, 100);
    let len = envelopes.len();
    let start_total = Instant::now();
    let rw = database.rw_transaction().unwrap();
    for e in envelopes {
        rw.insert(e).unwrap();
    }
    rw.commit().unwrap();
    // for chunk in envelopes.into_iter().chunks(10000).into_iter() {
    //     let rw = database.rw_transaction().unwrap();
    //     for c in chunk {
    //         rw.insert(c).unwrap();
    //     }
    //     rw.commit().unwrap();
    // }

    let elapsed_total = start_total.elapsed();
    println!("All {} envelopes inserted in {:?}", len, elapsed_total);

    let metadata = fs::metadata(temp_path).unwrap();
    let file_size = metadata.len();
    println!(
        "Database file size: {} bytes ({:.2} MB)",
        file_size,
        file_size as f64 / (1024.0 * 1024.0)
    );
}
