use crate::parser::Statement;
use crate::parser::Expr;
use crate::parser::Prefix;
use crate::parser::Operator;

#[derive(Debug, PartialEq)]
pub enum Object {
    Integer(i32),
    Boolean(bool),
    Null,
    Return(Box<Object>),
}

fn eval_expr(expression: Expr) -> Object {
    match expression {
        Expr::Const(num) => Object::Integer(num),
        Expr::Boolean(val) => Object::Boolean(val),
        Expr::Prefix { prefix: Prefix::Bang, value: expr } => {
            match eval_expr(*expr) {
                Object::Boolean(val) => Object::Boolean(!val),
                _ => panic!("! operator only valid for boolean type"),
            }
        },
        Expr::Prefix { prefix: Prefix::Minus, value: expr } => {
            match eval_expr(*expr) {
                Object::Integer(val) => Object::Integer(-val),
                _ => panic!("minus operator only valid for integer type"),
            }
        },
        Expr::Infix { left, operator: Operator::Plus, right } => {
            match (eval_expr(*left), eval_expr(*right)) {
                (Object::Integer(left), Object::Integer(right)) => Object::Integer(left + right),
                _ => panic!("plus operator only valid on integer types")
            }
        },
        Expr::Infix { left, operator: Operator::Minus, right } => {
            match (eval_expr(*left), eval_expr(*right)) {
                (Object::Integer(left), Object::Integer(right)) => Object::Integer(left - right),
                _ => panic!("minus operator only valid on integer types")
            }
        },
        Expr::Infix { left, operator: Operator::Multiply, right } => {
            match (eval_expr(*left), eval_expr(*right)) {
                (Object::Integer(left), Object::Integer(right)) => Object::Integer(left * right),
                _ => panic!("multiply operator only valid on integer types")
            }
        },
        Expr::Infix { left, operator: Operator::Divide, right } => {
            match (eval_expr(*left), eval_expr(*right)) {
                (Object::Integer(left), Object::Integer(right)) => Object::Integer(left / right),
                _ => panic!("divide operator only valid on integer types")
            }
        },
        Expr::Infix { left, operator: Operator::LessThan, right } => {
            match (eval_expr(*left), eval_expr(*right)) {
                (Object::Integer(left), Object::Integer(right)) => Object::Boolean(left < right),
                _ => panic!("less than operator only valid on integer types")
            }
        },
        Expr::Infix { left, operator: Operator::GreaterThan, right } => {
            match (eval_expr(*left), eval_expr(*right)) {
                (Object::Integer(left), Object::Integer(right)) => Object::Boolean(left > right),
                _ => panic!("greater than operator only valid on integer types")
            }
        },
        Expr::Infix { left, operator: Operator::Equals, right } => {
            match (eval_expr(*left), eval_expr(*right)) {
                (Object::Integer(left), Object::Integer(right)) => Object::Boolean(left == right),
                (Object::Boolean(left), Object::Boolean(right)) => Object::Boolean(left == right),
                _ => panic!("equals operator used on invalid types")
            }
        },
        Expr::Infix { left, operator: Operator::NotEquals, right } => {
            match (eval_expr(*left), eval_expr(*right)) {
                (Object::Integer(left), Object::Integer(right)) => Object::Boolean(left != right),
                (Object::Boolean(left), Object::Boolean(right)) => Object::Boolean(left != right),
                _ => panic!("not equals operator used on invalid types")
            }
        },
        Expr::If { condition, consequence, alternative } => {
            if eval_expr(*condition) == Object::Boolean(true) {
                eval_statements(consequence)
            } else {
                eval_statements(alternative)
            }
        },
        _ => panic!("eval expr not implemented for this type")
    }
}

fn eval_statement(statement: Statement) -> Object {
    match statement {
        Statement::Expression(expr) => eval_expr(expr),
        Statement::Return{value: expr} => Object::Return(Box::new(eval_expr(expr))),
        _ => panic!("unsupported statement type"),
    }
}

fn eval_statements(statements: Vec<Statement>) -> Object {
    let mut result = Object::Null;

    for statement in statements {
        result = eval_statement(statement);

        if let &Object::Return(_) = &result {
            return result;
        }
    }

    result
}

pub fn eval_program(statements: Vec<Statement>) -> Object {
    let result = eval_statements(statements);

    // if object is return type, unwrap it
    if let &Object::Return(_) = &result {
        match result {
            Object::Return(res) => return *res,
            _ => unreachable!(),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lexer;
    use crate::parser::parse;

    #[test]
    fn eval_int_literal() {
        test_eval("5;", Object::Integer(5));
    }

    #[test]
    fn eval_bool() {
        test_eval("true;", Object::Boolean(true));
        test_eval("false;", Object::Boolean(false));
    }

    #[test]
    fn eval_bang() {
        test_eval("!true;", Object::Boolean(false));
        test_eval("!false;", Object::Boolean(true));
        test_eval("!(1 > 2);", Object::Boolean(true));
    }

    #[test]
    fn eval_negative() {
        test_eval("-5;", Object::Integer(-5));
        test_eval("-(1 - 2);", Object::Integer(1));
    }

    #[test]
    fn eval_infix() {
        test_eval("5 + 5;", Object::Integer(10));
        test_eval("5 - 5;", Object::Integer(0));
        test_eval("5 * 5;", Object::Integer(25));
        test_eval("5 / 5;", Object::Integer(1));
        test_eval("5 > 1;", Object::Boolean(true));
        test_eval("5 < 1;", Object::Boolean(false));
        test_eval("5 == 1;", Object::Boolean(false));
        test_eval("5 != 1;", Object::Boolean(true));
        test_eval("true == true;", Object::Boolean(true));
        test_eval("true != true;", Object::Boolean(false));
        test_eval("(1 > 2) == false;", Object::Boolean(true));
    }

    #[test]
    fn eval_infix_nested_types() {
        test_eval("(1 + 2) + 3;", Object::Integer(6));
        test_eval("(1 + 2) - 3;", Object::Integer(0));
        test_eval("(1 + 2) * 3;", Object::Integer(9));
        test_eval("(1 + 2) / 3;", Object::Integer(1));
        test_eval("(1 + 2) < 3;", Object::Boolean(false));
        test_eval("(1 + 2) > 3;", Object::Boolean(false));
        test_eval("(1 > 2) == false;", Object::Boolean(true));
        test_eval("(1 > 2) != false;", Object::Boolean(false));
    }

    #[test]
    fn eval_if() {
        test_eval("if (true) { 10; };", Object::Integer(10));
        test_eval("if (false) { 10; };", Object::Null);
        test_eval("if (false) { 10; } else { 11; };", Object::Integer(11));
        test_eval("if (1 > 2) { 10; } else { 11; };", Object::Integer(11));
        test_eval("if (1 < 2) { 10; } else { 11; };", Object::Integer(10));
    }

    #[test]
    fn eval_return() {
        test_eval("return 10;", Object::Integer(10));
        test_eval("return 10; 11;", Object::Integer(10));
        test_eval("9; return 2 * 5; 9;", Object::Integer(10));
        test_eval(r#"
            if (10 > 1) {
              if (10 > 1) {
                return 10;
              };

              return 1;
            };
        "#, Object::Integer(10));
    }

//    #[test]
//    fn eval_binding() {
//        test_eval("let a = 10; a;", Object::Integer(10));
//    }

    fn test_eval(input: &str, expected: Object) {
        let mut tokens = lexer().parse(input.as_bytes()).unwrap();
        let ast = parse(&mut tokens);
        let obj = eval_program(ast);

        assert_eq!(
            expected,
            obj
        );
    }
}
