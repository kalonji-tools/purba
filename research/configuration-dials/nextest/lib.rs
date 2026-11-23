//! Tests that research/configuration-dials/nextest/run.sh runs under each dial.

#[cfg(test)]
mod tests {
    #[test]
    fn sleeps_three_seconds() {
        std::thread::sleep(std::time::Duration::from_secs(3));
        assert!(true, "the test only sleeps");
    }

    #[test]
    fn fails_on_its_first_attempt() {
        let attempt = std::env::var("NEXTEST_ATTEMPT").unwrap_or_default();
        assert_ne!(attempt, "1", "the first attempt fails, a retry passes");
    }

    #[test]
    #[ignore = "probe: an ignored test that fails"]
    fn ignored_and_fails() {
        assert!(false, "an ignored test that fails");
    }
}
