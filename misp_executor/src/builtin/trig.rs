use crate::{Error, Executor, Value, config::AngleMode};

macro_rules! trig_op {
    ($name:ident, $op_name:literal, $op:tt) => {
        pub fn $name(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
            if args.len() != 1 {
                return Err(Error::FunctionArity {
                    name: $op_name.to_string(),
                    expected: 1,
                    actual: args.len(),
                });
            }

            let arg = executor.eval(&args[0])?;
            let value_f64: f64 = match &arg {
                Value::Decimal(d) => d.to_f64().unwrap(),
                _ => return Err(Error::FunctionCall),
            };

            let proper_angle = match executor.config.angle_mode {
                AngleMode::Radians => value_f64,
                AngleMode::Degrees => value_f64.to_radians(),
            };

            let result = proper_angle.$op();
            Ok(Value::Decimal(BigDecimal::from_f64(result).unwrap()))
        }
    };
}

macro_rules! inverse_trig_op {
    ($name:ident, $op_name:literal, $op:tt) => {
        pub fn $name(executor: &mut Executor, args: &[Value]) -> Result<Value, Error> {
            if args.len() != 1 {
                return Err(Error::FunctionArity {
                    name: $op_name.to_string(),
                    expected: 1,
                    actual: args.len(),
                });
            }

            let arg = executor.eval(&args[0])?;

            let value_f64: f64 = match &arg {
                Value::Decimal(d) => d.to_f64().unwrap(),
                _ => return Err(Error::FunctionCall),
            };

            let result_radians = value_f64.$op();

            let proper_angle = match executor.config.angle_mode {
                AngleMode::Radians => result_radians,
                AngleMode::Degrees => result_radians.to_degrees(),
            };

            Ok(Value::Decimal(BigDecimal::from_f64(proper_angle).unwrap()))
        }
    };
}

// trig_op!(builtin_sin, "sin", sin);
// trig_op!(builtin_cos, "cos", cos);
// trig_op!(builtin_tan, "tan", tan);

// inverse_trig_op!(builtin_asin, "asin", asin);
// inverse_trig_op!(builtin_acos, "acos", acos);
// inverse_trig_op!(builtin_atan, "atan", atan);
