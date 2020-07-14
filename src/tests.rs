#[macro_use]
#[cfg(test)]
mod tests {
    use super::*;

    use tax::get_42;

    #[test]
    fn test_add() {
        assert_eq!(
            get_cmd_rank_arg(vec!("one".to_string(), "two".to_string(), "3".to_string())),
            Ok(Some(3))
        );
    }

    #[test]
    fn test_get_42() {
        assert_eq!(tax::get_42(), 42)
    }
}
