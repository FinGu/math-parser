use mp::math_parser::*;

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn infix_to_postfix_valid1() {
        let t = infix_to_postfix::new("((8*21)+89/14)^4").parse().unwrap();

        assert_eq!(t, "8 21 * 89 14 / + 4 ^");
    }

    #[test]
    fn infix_to_postfix_valid2() {
        let t = infix_to_postfix::new("50 * -45").parse().unwrap();

        assert_eq!(t, "50 45 m *");
    }

    #[test]
    #[should_panic]
    fn infix_to_postfix_failed1() {
        infix_to_postfix::new("50 **/ (-45)").parse().unwrap();
    }

    #[test]
    #[should_panic]
    fn infix_to_postfix_failed2() {
        infix_to_postfix::new("(5 + 5").parse().unwrap();
    }

    #[test]
    fn postfix_to_result_valid1() {
        let t = postfix_to_result::new("8 21 * 89 14 / + 2 ^")
            .parse(false)
            .unwrap();

        assert_eq!(t, "30400.414"); //f32 precision
    }

    #[test]
    fn postfix_to_result_valid2() {
        let t = postfix_to_result::new("50 45 m *").parse(false).unwrap();

        assert_eq!(t, "-2250");
    }
    #[test]
    #[should_panic]
    fn postfix_to_result_failed1() {
        postfix_to_result::new("50 45 + -").parse(false).unwrap();
    }

    #[test]
    #[should_panic]
    fn postfix_to_result_failed2() {
        postfix_to_result::new("+").parse(false).unwrap();
    }
}
