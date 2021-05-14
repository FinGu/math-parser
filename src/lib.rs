#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub mod math_parser {
    use math_parser_errors::*;

    type stack<T> = Vec<T>;

    type deque<T> = std::collections::VecDeque<T>;

    pub type math_parser_result<T> = Result<T, math_parser_errors>;

    #[derive(Debug, Clone, Copy)]
    pub enum math_parser_errors {
        mismatched_parenthesis,
        invalid_notation,
        invalid_op,
    }

    impl std::fmt::Display for math_parser_errors {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    invalid_notation => "Invalid notation",
                    mismatched_parenthesis => "Mismatched parenthesis",
                    invalid_op => "Invalid operator",
                }
            )
        }
    }

    #[derive(PartialEq, PartialOrd)]
    enum assoc {
        right,
        left,
    }
    #[derive(PartialEq, PartialOrd)]
    struct math_operator {
        operator: char,
        precedence: i32,
        associativity: assoc,
    }

    const BASE: u32 = 10;
    const SPACE: char = ' ';
    const LP: char = '(';

    impl math_operator {
        fn new(opchar: char) -> Option<Self> {
            let (preced, assoc) = match opchar {
                '+' | '-' => (2, assoc::left),

                'p' | 'm' => (2, assoc::right), // unary ( m = minus, p = plus)

                '*' | '/' => (3, assoc::left),

                '^' => (4, assoc::right),

                LP => (-1, assoc::left), // extra

                _ => return None,
            };

            Some(math_operator {
                operator: opchar,
                precedence: preced,
                associativity: assoc,
            })
        }

        fn function(&self, first: f32, second: f32) -> math_parser_result<f32> {
            Ok(match self.operator {
                '+' => first + second,
                '-' => first - second,
                '*' => first * second,
                '/' => first / second,
                '^' => first.powf(second),
                'm' => -1.0 * first,
                'p' => 1.0 * first,
                _ => return Err(invalid_op),
            })
        }

        fn is_real_op(&self) -> bool {
            !matches!(self.operator, LP | 'p' | 'm')
        }
    }

    pub struct math_parser;

    impl math_parser {
        pub fn parse(notation: &str, dbg: bool) -> math_parser_result<f32> {
            let itop = infix_to_postfix::new(notation).parse()?;

            let ptor = postfix_to_result::new(&itop).parse(dbg)?;

            Ok(ptor.parse::<f32>().unwrap())
        }
    }

    pub struct infix_to_postfix {
        input_str: String,

        output_queue: deque<char>,

        operator_stack: stack<math_operator>,
    }

    //SHUNTING YARD
    impl infix_to_postfix {
        pub fn new(input: &str) -> Self {
            Self {
                input_str: input.to_string(),
                output_queue: deque::new(),
                operator_stack: stack::new(),
            }
        }

        fn handle_token(&mut self, token: char) {
            let opstack = &mut self.operator_stack;

            let outqueue = &mut self.output_queue;

            if token.is_digit(BASE) {
                // is base-10 number
                outqueue.push_back(token);
            } else if let Some(operator) = math_operator::new(token) {
                // is valid operator ( functions not implemented )
                // https://github.com/rust-lang/rust/issues/53667
                while matches!(opstack.last(), Some(last_operator) if
                    ((last_operator.precedence == operator.precedence && operator.associativity == assoc::left)
                    || (last_operator.precedence > operator.precedence)) // if LP was allowed, this would always be true
                    && operator.is_real_op()
                ) {
                    /*
                    input => (1+1)*2^1
                    ...
                    (2 != -1)
                    stack => ( +
                    queue => 1 1
                    ... (loops till ) found )
                    stack =>
                    queue => 1 1 +
                    ... ( push the rest )
                    stack => * ^
                    queue => 1 1 + 2 1
                    ... (input ended)
                    queue => 1 1 + 2 1 ^ *
                    */
                    outqueue.push_back(SPACE); // for valid formatting
                    outqueue.push_back(opstack.pop().unwrap().operator);
                }

                if operator.is_real_op() {
                    // LP is always removed, but the extra space isnt
                    outqueue.push_back(SPACE);
                }

                opstack.push(operator);
            } else if token == ')' {
                while let Some(last_operator) = opstack.last() {
                    if last_operator.operator == LP {
                        opstack.pop(); //discard all LP
                        continue;
                    }

                    outqueue.push_back(SPACE);
                    outqueue.push_back(opstack.pop().unwrap().operator); //pop op to queue
                }
            } else if token == '.' || token == ',' {
                outqueue.push_back('.'); //f32 parse cannot handle ,
            }
        }

        fn clean_stack(&mut self) -> math_parser_result<()> {
            while !self.operator_stack.is_empty() {
                let popd = self.operator_stack.pop().unwrap().operator;

                if popd == LP {
                    //RPs arent pushed so it doesnt make sense to check them
                    return Err(mismatched_parenthesis);
                }

                // while there are tokens left, pop the rest of the operators into the queue

                self.output_queue.push_back(SPACE);
                self.output_queue.push_back(popd);
            }
            Ok(())
        }

        pub fn parse(&mut self) -> math_parser_result<String> {
            let char_str: String = self.input_str.split_whitespace().collect(); //removing spaces for the unary_handle function

            let mut char_stack = char_str.chars().collect();

            self.unary_handle(&mut char_stack)?;

            char_stack.into_iter().for_each(|c| self.handle_token(c));

            self.clean_stack()?;

            self.to_string(&self.output_queue)
        }

        fn to_string(&self, outqueue: &deque<char>) -> math_parser_result<String> {
            let outstr: String = outqueue.iter().collect();

            if outstr.contains("  ") {
                // notation is invalid in case of two spaces
                return Err(invalid_notation);
            }

            Ok(outstr)
        }

        fn unary_handle(&self, chrs: &mut stack<char>) -> math_parser_result<()> {
            for i in 0..chrs.len() {
                let (left, right) = chrs.split_at_mut(i);
                //splits the array [1,+,1] (i = 1) => ([1], [+,1])

                let token = *right.first().unwrap(); //chrs[i]

                if matches!(math_operator::new(token), Some(tok_as_op) if tok_as_op.is_real_op()) &&
                    matches!(right.get(1), Some(nxt) if nxt.is_digit(BASE)) && // (2+1)-(2+1), if the next char isnt a digit, it doesnt make sense, PS : 0 is i, so 1 is i+1
                    matches!(if i == 0 { right.first() } else { left.last() }, Some(bhd) // the entire check is ignored in case i is 0
                    if *bhd != ')' // (2+1)-4, the minus in this case would be considered unary without this check
                    && !bhd.is_digit(BASE))
                // 2+1-4, behind must not be a digit, or else an operation would be turned to unary
                {
                    *right.first_mut().unwrap() = match token {
                        //mut chrs[i]
                        '-' => 'm',
                        '+' => 'p',
                        _ => return Err(invalid_notation),
                    }
                }
            }
            Ok(())
        }
    }

    pub struct postfix_to_result {
        input_str: String,

        output_stack: stack<String>,
    }

    impl postfix_to_result {
        pub fn new(input: &str) -> Self {
            Self {
                input_str: input.to_string(),
                output_stack: stack::new(),
            }
        }

        fn handle_token(&mut self, token: String, dbg: bool) -> math_parser_result<()> {
            let outstack = &mut self.output_stack;

            if token.parse::<f32>().is_ok() {
                outstack.push(token);
            } else if let Some(operator) = math_operator::new(token.chars().next().unwrap()) {
                let mut arg1 = std::f32::NAN;

                let mut pop_num_or = |err| -> math_parser_result<f32> {
                    if let Some(out) = outstack.pop() {
                        Ok(out.parse::<f32>().map_err(|_| err)?)
                    } else {
                        Err(err)
                    }
                }; //pops val and convert to f32 or returns err

                let arg2 = pop_num_or(invalid_notation)?;

                let result = if operator.is_real_op() {
                    // real ops need another arg
                    arg1 = pop_num_or(invalid_notation)?;

                    operator.function(arg1, arg2)
                } else {
                    //unary ops dont need a sec arg, arg1 is NaN
                    operator.function(arg2, arg1)
                }
                .map_err(|e| e)?
                .to_string();

                /*
                input => 3 2 -
                ...
                input = -
                stack => 3 2
                ...
                (pop1 = 2, pop2 = 3) (order changed due to it being a stack)
                (first = pop2, second = pop1)
                ...
                stack = 1
                */

                if dbg {
                    println!("{} {} {} = {}", arg1, operator.operator, arg2, result);
                } //TODO : return an array with those instead of printing

                outstack.push(result);
            }
            Ok(())
        }

        pub fn parse(&mut self, dbg: bool) -> math_parser_result<String> {
            let cloned_input = self.input_str.clone();

            let _l = cloned_input.split_whitespace();

            for c in _l {
                self.handle_token(c.to_owned(), dbg)?;
            }

            Ok(self.output_stack.join(""))
        }
    }
}
