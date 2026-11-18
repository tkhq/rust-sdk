use crate::output::Shell;
use std::io::Write;
use turnkey_proofs::tvc::Rejected;

/// Best-effort human status. The acceptance result still controls finalization.
pub(crate) fn present_status<W: Write, E: Write, T>(
    shell: &mut Shell<W, E>,
    acceptance: &Result<T, Rejected>,
) {
    if acceptance.is_ok() {
        let _ = shell.human().status("Proof verification", "passed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::TestShell;

    #[test]
    fn success_status_is_human_only() {
        let acceptance: Result<(), Rejected> = Ok(());
        let mut human = TestShell::with_human_formatter();
        present_status(&mut human, &acceptance);
        assert_eq!(human.into_stderr(), b"Proof verification: passed\n");

        let mut json = TestShell::with_json_formatter();
        present_status(&mut json, &acceptance);
        assert!(json.into_stderr().is_empty());
        let mut json = TestShell::with_json_formatter();
        present_status(&mut json, &acceptance);
        assert!(json.into_stdout().is_empty());
    }

    #[derive(Default)]
    struct BrokenWriter;
    impl Write for BrokenWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn failed_status_delivery_does_not_replace_acceptance() {
        let acceptance: Result<(), Rejected> = Ok(());
        let mut shell: Shell<Vec<u8>, BrokenWriter> = Shell::default();
        present_status(&mut shell, &acceptance);
        assert!(acceptance.is_ok());
    }
}
