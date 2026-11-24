// Rust has no tag. "slow" and "db" are parts of the test names.
#[cfg(test)]
mod math {
    #[test]
    fn add() {
        assert_eq!(1 + 1, 2, "addition");
    }

    #[test]
    fn add_slow() {
        assert_eq!(2 + 2, 4, "addition, slow");
    }
}

#[cfg(test)]
mod store {
    #[test]
    fn query_db() {
        assert!(true, "query");
    }

    #[test]
    fn migrate_db_slow() {
        assert!(true, "migrate");
    }
}
