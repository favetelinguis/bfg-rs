use std::fmt::{Display, Formatter};
use crate::TradeResultView;
use std::env::home_dir;
use chrono::Utc;
use tokio::io::AsyncWriteExt;
use bfg_core::decider::TradeResult;

pub fn write_results_to_file(result: TradeResult) {
    tokio::spawn(async move {
        let todays_file = Utc::now().date().to_string();
        let mut path = home_dir().expect("always have a home");
        path.push(format!("bfg/demo/{}.csv", todays_file));
        if let Ok(mut file)  = tokio::fs::OpenOptions::new()
            .append(true).create_new(true).open(path.as_path()).await {
            let headers = "epic,size,reference,wanted_entry_level,entry_time,actual_entry_level,exit_time,exit_level,opening_range_size,strategy_version";
            let initial_write = format!("{}\r\n{}", headers, to_csv(result));
            file.write(initial_write.as_bytes()).await;
        } else {
            let mut file =  tokio::fs::OpenOptions::new()
                .append(true).create(false).open(path.as_path()).await.expect("we know file exists here");
            file.write(to_csv(result).as_bytes()).await;
        }
    });
}

    fn to_csv(result: TradeResult) -> String {
        let wanted_entry_level = format!("{:.1}", result.wanted_entry_level);
        let actual_entry_level = format!("{:.1}", result.actual_entry_level);
        let exit_level = format!("{:.1}", result.exit_level);
        let reference = format!("{:?}", result.reference);
        format!("{},{},{},{},{},{},{},{},{},{}\r\n", result.epic, result.size, reference, wanted_entry_level, result.entry_time, actual_entry_level, result.exit_time, exit_level, result.opening_range_size, result.strategy_version)
    }

#[cfg(test)]
mod tests {
    #[tokio::test]
    // #[test]
    async fn it_works_factory() {
    }
}
