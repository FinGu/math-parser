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
        invalid_function,
    }

    impl std::fmt::Display for math_parser_errors {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    mismatched_parenthesis => "Mismatched parenthesis",
                    invalid_notation => "Invalid notation",
                    invalid_function => "Invalid function",
                }
            )
        }
    }

    type Ftype = f32;
    const BASE: u32 = 10;
    const SPACE: char = ' ';
    const LP: char = '(';

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

    impl math_operator {
        fn new(opchar: char) -> Option<Self> {
            let (preced, assoc) = match opchar {
                '+' | '-' => (2, assoc::left),

                '*' | '/' => (3, assoc::left),

                '^' => (4, assoc::right),

                'p' | 'm' => (5, assoc::left), // unary ( m = minus, p = plus )

                LP => (-1, assoc::left), // extra

                _ => return None,
            };

            Some(math_operator {
                operator: opchar,
                precedence: preced,
                associativity: assoc,
            })
        }

        fn solve(&self, first: Ftype, second: Ftype) -> math_parser_result<Ftype> {
            Ok(match self.operator {
                '+' => first + second,
                '-' => first - second,
                '*' => first * second,
                '/' => first / second,
                '^' => first.powf(second),
                'm' => -1.0 * first,
                'p' => 1.0 * first,
                _ => return Err(invalid_notation),
            })
        }

        fn is_real_op(&self) -> bool {
            matches!(self.operator, '+' | '-' | '*' | '/' | '^')
        }

        fn is_function(&self) -> bool {
            match self.operator {
                'p' | 'm' => false,
                op => op.is_alphabetic() || op == '!',
            }
        }
    }

    struct math_function<'a>(&'a str, usize);

    impl<'a> math_function<'a> {
        fn new(inp: &'a str) -> Option<Self> {
            Some(Self(
                inp,
                match inp {
                    "sin" | "cos" | "tan" => 1,
                    "log" => 2,
                    _ => return None,
                },
            ))
        }

        fn solve(&self, first: Ftype, second: Ftype) -> math_parser_result<Ftype> {
            Ok(match self.0 {
                "sin" => first.sin(),
                "cos" => first.cos(),
                "tan" => first.tan(),
                "log" => first.log(second),
                _ => return Err(invalid_function),
            })
        }
    }

    pub struct math_parser;

    impl math_parser {
        pub fn parse(notation: &str) -> math_parser_result<(Ftype, stack<String>)> {
            let itop = infix_to_postfix.parse(notation)?;

            let mut dbg = stack::new();

            let ptor = postfix_to_result.parse(&itop, &mut Some(&mut dbg))?;

            match ptor.parse::<Ftype>() {
                Ok(out) => Ok((out, dbg)),
                Err(_) => Err(invalid_notation),
            }
        }
    }

    pub struct infix_to_postfix;

    //naive shunting yard
    impl infix_to_postfix {
        fn handle_token(
            &self,
            opstack: &mut stack<math_operator>,
            outqueue: &mut deque<char>,
            token: char,
        ) {
            if token.is_digit(BASE) || token == ',' || token == '.' {
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
                    let popd = opstack.pop().unwrap();

                    if !popd.is_function() {
                        //makes sure the popped op isnt part of a function
                        outqueue.push_back(SPACE);
                    }

                    outqueue.push_back(popd.operator);
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
                        break;
                    }

                    if !last_operator.is_function() {
                        // abc cant be turned into a b c
                        outqueue.push_back(SPACE);
                    }

                    outqueue.push_back(opstack.pop().unwrap().operator); //pop op to queue
                }
            } else if token.is_alphabetic() || token == '!' {
                //is function
                opstack.push(math_operator {
                    operator: token,
                    precedence: 4,
                    associativity: assoc::right,
                });
            }
        }

        fn clean_stack(
            &self,
            opstack: &mut stack<math_operator>,
            outqueue: &mut deque<char>,
        ) -> math_parser_result<()> {
            while !opstack.is_empty() {
                let popd = opstack.pop().unwrap();

                if popd.operator == LP {
                    //RPs arent pushed so it doesnt make sense to check them
                    return Err(mismatched_parenthesis);
                }

                // while there are tokens left, pop the rest of the operators into the queue

                if !popd.is_function() {
                    outqueue.push_back(SPACE);
                }

                outqueue.push_back(popd.operator);
            }
            Ok(())
        }

        pub fn parse(&self, input_str: &str) -> math_parser_result<String> {
            let mut operator_stack: stack<math_operator> = stack::new();

            let mut output_queue: deque<char> = deque::new();

            let char_str: String = input_str.split_whitespace().collect(); //removing spaces for the unary_handle function

            let mut char_stack = char_str.chars().collect();

            self.unary_handle(&mut char_stack)?;

            char_stack
                .into_iter()
                .for_each(|c| self.handle_token(&mut operator_stack, &mut output_queue, c));

            self.clean_stack(&mut operator_stack, &mut output_queue)?;

            self.to_string(&output_queue)
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

    pub struct postfix_to_result;

    impl postfix_to_result {
        fn handle_token(
            &self,
            outstack: &mut stack<String>,
            token: String,
            dbg: Option<&mut stack<String>>,
        ) -> math_parser_result<()> {
            if token.parse::<Ftype>().is_ok() || token.contains(',') {
                outstack.push(token);
            } else if let Some(operator) = math_operator::new(token.chars().next().unwrap()) {
                let mut arg1 = Ftype::NAN;

                let pop_num_or = |err, stack: &mut stack<String>| -> math_parser_result<Ftype> {
                    match stack.pop() {
                        Some(out) => Ok(out.parse::<Ftype>().map_err(|_| err)?),
                        None => Err(err),
                    }
                }; //pops val and convert to Ftype or returns err

                let arg2 = pop_num_or(invalid_notation, outstack)?;

                let result = if operator.is_real_op() {
                    arg1 = pop_num_or(invalid_notation, outstack)?;
                    // 1 2
                    // is op +

                    // arg2 = pop(2)
                    // 1

                    //arg1 = pop(1)

                    operator.solve(arg1, arg2)
                } else {
                    //unary handler
                    operator.solve(arg2, arg1)
                }
                .map_err(|e| e)?
                .to_string();

                if let Some(dbgstck) = dbg {
                    dbgstck.push(format!(
                        "{} {} {} = {}",
                        arg1, operator.operator, arg2, result
                    ));
                }

                outstack.push(result);
            }

            Ok(())
        }

        fn handle_function(
            &self,
            outstack: &mut stack<String>,
            token: String,
            dbg: Option<&mut stack<String>>,
        ) -> math_parser_result<()> {
            outstack.push(" ".to_owned()); //pushes a &str space to separate args

            if !token.contains('!') || token.ends_with('!') {
                outstack.push(token);
                return Ok(());
            }

            fn next_or<'a>(
                err: math_parser_errors,
                iter: &mut dyn Iterator<Item = &'a str>,
            ) -> math_parser_result<&'a str> {
                match iter.next() {
                    Some(out) => Ok(out),
                    None => Err(err),
                } //gets next val or returns err
            }

            let as_ftype_or = |err, item: &str| -> math_parser_result<Ftype> {
                Ok(item.parse::<Ftype>().map_err(|_| err)?)
            }; // parses to ftype

            //ex input : 10,10!gol
            //func input : 10,10
            //func : log!

            let func_name: String = token
                .chars()
                .rev()
                .take_while(|x| x.is_alphabetic())
                .collect(); //gets 'log'

            let func = match math_function::new(&func_name) {
                Some(func) => func,
                None => return Err(invalid_function),
            }; // the math func ( log )

            let func_args = next_or(invalid_function, &mut token.split("!"))?; //the args ( 10,10 )

            let mut func_args = func_args.split(","); //[10, 10]

            let arg1 = as_ftype_or(invalid_notation, next_or(invalid_function, &mut func_args)?)?;

            let arg2 = if func.1 == 2 {
                //if is 2 arg func
                as_ftype_or(invalid_notation, next_or(invalid_function, &mut func_args)?)?
            } else {
                Ftype::NAN
            };

            let result = func.solve(arg1, arg2).map_err(|e| e)?.to_string();

            if let Some(dbgstck) = dbg {
                dbgstck.push(format!("{} {} {} = {}", func.0, arg1, arg2, result));
            }

            outstack.push(result);

            Ok(())
        }

        pub fn parse(
            &self,
            input_str: &str,
            dbg: &mut Option<&mut stack<String>>,
        ) -> math_parser_result<String> {
            let mut output_stack: stack<String> = stack::new();

            input_str
                .split_whitespace() //split with whitespaces
                .map(
                    |str| {
                        self.handle_function(&mut output_stack, str.to_owned(), dbg.as_deref_mut())
                    }, //handles func tokens
                )
                .collect::<math_parser_result<()>>()?; //fail with collect

            let input_str = output_stack.into_iter().collect::<String>();

            output_stack = stack::new(); //new opstack for token handling

            input_str
                .split_whitespace()
                .map(|str| self.handle_token(&mut output_stack, str.to_owned(), dbg.as_deref_mut()))
                .collect::<math_parser_result<()>>()?;

            Ok(output_stack.join("")) //stack<String> to String
        }
    }
}
