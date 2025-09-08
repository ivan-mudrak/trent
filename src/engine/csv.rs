use crate::engine::core::TxEngine;
use crate::readers::csv::{csv_buf_reader_for_file, read_csv_from_file};
use crate::types::{client::ClientTx, error::Result};

pub fn process_csv_file<W: std::io::Write>(file_path: &str, wrt: W) -> Result<()> {
    let metadata = std::fs::metadata(file_path)?;
    let file_size = metadata.len();

    if file_size < 10 * 1024 * 1024 {
        process_csv_file_single_thread(file_path, wrt)
    } else {
        process_csv_file_multi_thread(file_path, wrt)
    }
}

fn process_csv_file_multi_thread<W: std::io::Write>(file_path: &str, wrt: W) -> Result<()> {
    let mut engine = TxEngine::multi_threaded();

    let mut reader = csv_buf_reader_for_file(file_path)?;

    for row in reader.deserialize::<ClientTx>() {
        match row {
            Ok(tx) => {
                let _ = engine.process(vec![tx]);
            }
            Err(_) => {
                // TODO: log errors
            }
        }
    }

    let clients = engine.flush()?;

    clients.write_to_csv(wrt)?;
    Ok(())
}

fn process_csv_file_single_thread<W: std::io::Write>(file_path: &str, wrt: W) -> Result<()> {
    let mut engine = TxEngine::single_threaded();

    let txs = read_csv_from_file(file_path)?;

    let _ = engine.process(txs);

    let clients = engine.flush()?;

    clients.write_to_csv(wrt)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::readers::csv::read_csv_from_buffer;
    use crate::types::client::ClientAccount;

    #[test]
    fn single_vs_multi() {
        let file_path = "tests/single_vs_multi/transactions.csv";
        let mut output = Vec::new();
        process_csv_file_single_thread(file_path, &mut output).unwrap();
        let mut single_thread_output = read_csv_from_buffer::<ClientAccount>(&output).unwrap();

        output.clear();

        process_csv_file_multi_thread(file_path, &mut output).unwrap();
        let mut multi_thread_output = read_csv_from_buffer::<ClientAccount>(&output).unwrap();

        single_thread_output.sort_by_key(|acc| acc.client);
        multi_thread_output.sort_by_key(|acc| acc.client);

        assert_eq!(single_thread_output, multi_thread_output)
    }

    fn run_test_case(test_case: &str) {
        let mut output = Vec::new();
        process_csv_file(
            format!("tests/{test_case}/transactions.csv").as_str(),
            &mut output,
        )
        .unwrap();
        let mut output = read_csv_from_buffer::<ClientAccount>(&output).unwrap();

        let mut expected =
            read_csv_from_file::<ClientAccount>(format!("tests/{test_case}/accounts.csv").as_str())
                .unwrap();

        output.sort_by_key(|acc| acc.client);
        expected.sort_by_key(|acc| acc.client);

        assert_eq!(output, expected, "{}", test_case)
    }

    #[test]
    fn simple_settle_tx() {
        run_test_case("simple_settle_tx")
    }

    #[test]
    fn not_enough_funds() {
        run_test_case("not_enough_funds")
    }

    #[test]
    fn double_dispute() {
        run_test_case("double_dispute")
    }

    #[test]
    fn double_resolve() {
        run_test_case("double_resolve")
    }

    #[test]
    fn try_withdraw_when_all_under_dispute() {
        run_test_case("try_withdraw_when_all_under_dispute")
    }

    #[test]
    fn try_deposit_when_locked() {
        run_test_case("try_deposit_when_locked")
    }

    #[test]
    fn try_withdraw_when_locked() {
        run_test_case("try_withdraw_when_locked")
    }
}
