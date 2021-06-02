use mp::math_parser::*;

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn infix_to_postfix_valid1() {
        let t = infix_to_postfix.parse("((8*21)+89/14)^4").unwrap();

        assert_eq!(t, "8 21 * 89 14 / + 4 ^");
    }

    #[test]
    fn infix_to_postfix_valid2() {
        let t = infix_to_postfix.parse("50 * -45").unwrap();

        assert_eq!(t, "50 45 m *");
    }

    #[test]
    #[should_panic]
    fn infix_to_postfix_failed1() {
        infix_to_postfix.parse("50 **/ (-45)").unwrap();
    }

    #[test]
    #[should_panic]
    fn infix_to_postfix_failed2() {
        infix_to_postfix.parse("(5 + 5").unwrap();
    }

    #[test]
    fn postfix_to_result_valid1() {
        let t = postfix_to_result
            .parse("8 21 * 89 14 / + 2 ^", &mut None)
            .unwrap();

        assert_eq!(t, "30400.414"); //f32 precision
    }

    #[test]
    fn postfix_to_result_valid2() {
        let t = postfix_to_result.parse("50 45 m *", &mut None).unwrap();

        assert_eq!(t, "-2250");
    }

    #[test]
    #[should_panic]
    fn postfix_to_result_failed1() {
        postfix_to_result.parse("50 45 + -", &mut None).unwrap();
    }

    #[test]
    #[should_panic]
    fn postfix_to_result_failed2() {
        postfix_to_result.parse("+", &mut None).unwrap();
    }

    #[test]
    fn math_parser_valid1() {
        let t = parse("log!10,10 + sin!1").unwrap();

        assert_eq!(t.0, 1.841471);
    }

    #[test]
    fn math_parser_valid2() {
        let t = parse("((2048 / 4) - 12) * log!100,10").unwrap(); //log of 100 on base 10

        assert_eq!(t.0, 1000.0);
    }

    #[test]
    #[should_panic]
    fn math_parser_failed1() {
        parse("log!-10,10 / 1").unwrap(); // no support for negative args in functions
    }

    #[test]
    #[should_panic]
    fn math_parser_failed2() {
        parse("928 / 2 +* 4").unwrap();
    }
}
