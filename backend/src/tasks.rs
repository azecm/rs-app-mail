use std::process::Command;

use crate::constants::path_to_temp_upload;
use crate::sse::sse_cleaner;

pub async fn run_tasks() {
    let mut interval_timer = tokio::time::interval(chrono::Duration::hours(1).to_std().unwrap());
    loop {
        interval_timer.tick().await;
        tokio::spawn(async {
            // закрываем отвалившиеся соединения
            sse_cleaner();

            // удаляем файлы из временной директории, которым более суток
            let temp_dir = path_to_temp_upload("");
            Command::new("sh")
                .arg("-c")
                .arg(&format!("find {temp_dir} -type f -ctime +1d -delete"))
                .output()
                .expect("failed to execute process");
        });
    }
}