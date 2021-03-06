use super::*;
use std::io::Write;

impl<'api, 'pin> Runner<'api, 'pin> {
    pub fn upgrade(&self, cmd: SubCommand) {
        debug!("Starting in upgrade");
        match cmd {
            SubCommand::SelfUpdate { check, download } => {
                if check && download {
                    eprintln!("Cannont check & download at the same time!");
                    process::exit(1);
                }
                let is_alfred_v3 = self.config.as_ref().unwrap().is_alfred_v3();
                if check {
                    if let Some(item) = self.get_upgrade_item() {
                        ::write_to_alfred(vec![item], is_alfred_v3);
                    } else {
                        let item =
                            alfred::ItemBuilder::new("Couldn't find a new update.").into_item();
                        ::write_to_alfred(vec![item], is_alfred_v3);
                    }
                } else if download {
                    let filename = self.updater.as_ref().unwrap().download_latest();
                    if let Ok(filename) = filename {
                        filename.to_str().map(|p| {
                            let _ = io::stdout()
                                .write(format!("Download successful: {}", p).as_bytes());
                        });
                    } else {
                        let _ =
                            io::stdout().write(b"Error: Couldn't download the latest workflow.");
                        process::exit(1);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
