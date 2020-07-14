pub mod tax {
    pub fn get_42() -> i32 {
        42
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_42() {
        assert_eq!(tax::get_42(), 42)
    }
}
