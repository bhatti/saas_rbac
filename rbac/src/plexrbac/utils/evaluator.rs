//#![crate_name = "doc"]

use chrono::{NaiveDate, Utc, Datelike};
use plexrbac::common::ValueWrapper;
use std::collections::HashMap;
use evalexpr::*;

////////////////////////////////////////////////////////////////////////////////
/// Defines helper method to evaluate boolean expression
///
pub fn evaluate(expr: &str, properties: &HashMap<String, ValueWrapper>) -> Result<bool, evalexpr::EvalexprError> {
    let mut ctx = HashMapContext::new();
    for (n, v) in properties {
        add_context_func(&mut ctx, n.as_str(), v.clone())?;
    }
    add_builtin(&mut ctx)?;
    match eval_with_context(expr, &ctx) {
        Ok(Value::Boolean(b)) => {
            Ok(b)
        },
        Ok(Value::Int(n)) => Err(EvalexprError::CustomMessage(format!("Invalid int expression {}, only boolean results are supported '{}'", n, expr))),
        Ok(Value::Float(f)) => Err(EvalexprError::CustomMessage(format!("Invalid float expression {}, only boolean results are supported '{}'", f, expr))),
        Ok(Value::String(s)) => Err(EvalexprError::CustomMessage(format!("Invalid string expression {}, only boolean results are supported '{}'", s, expr))),
        Ok(Value::Empty) => Err(EvalexprError::CustomMessage(format!("Invalid empty expression, only boolean results are supported '{}'", expr))),
        Err(e) => Err(EvalexprError::CustomMessage(format!("Invalid expression, only boolean results are supported '{}' --- {:?}", expr, e))),
        _ => Err(EvalexprError::CustomMessage(format!("Invalid expression, only boolean results are supported '{}' --- {:?}", expr, eval_with_context(expr, &ctx))))
    }
}

fn add_builtin(ctx: &mut HashMapContext) -> Result<bool, evalexpr::EvalexprError> { 
    if let Err(err) = ctx.set_function("geo_distance_km".to_string(),
         Function::new(
             Some(4),
             Box::new(|args| {
                 if let (Value::Float(lat1), Value::Float(lon1), Value::Float(lat2), Value::Float(lon2)) = (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone()) {
                    Ok(Value::Float(super::distance::distance_km(lat1, lon1, lat2, lon2)))
                 } else {
                     Err(EvalexprError::expected_number(args[0].clone()))
                 }
             }),
         ),
        ) {
        return Err(err)
    }
    if let Err(err) = ctx.set_function("regex_match".to_string(),
         Function::new(
             Some(2),
             Box::new(|args| {
                 if let (Value::String(rx), Value::String(s)) = (args[0].clone(), args[1].clone()) {
                    Ok(Value::Boolean(super::text::regex_match(rx.as_str(), s.as_str())))
                 } else {
                     Err(EvalexprError::expected_string(args[0].clone()))
                 }
             }),
         ),
        ) {
        return Err(err)
    }
    if let Err(err) = ctx.set_function("regex_find".to_string(),
         Function::new(
             Some(2),
             Box::new(|args| {
                 if let (Value::String(rx), Value::String(s)) = (args[0].clone(), args[1].clone()) {
                    Ok(Value::Boolean(super::text::regex_find(rx.as_str(), s.as_str())))
                 } else {
                     Err(EvalexprError::expected_string(args[0].clone()))
                 }
             }),
         ),
        ) {
        return Err(err)
    }
    if let Err(err) = ctx.set_function("current_year".to_string(),
         Function::new(
             None,
             Box::new(|_args| {
                 Ok(Value::Int(Utc::now().naive_utc().year() as i64))
             }),
         ),
        ) {
        return Err(err)
    }
    if let Err(err) = ctx.set_function("current_month".to_string(),
         Function::new(
             None,
             Box::new(|_args| {
                 Ok(Value::Int(Utc::now().naive_utc().month() as i64))
             }),
         ),
        ) {
        return Err(err)
    }
    if let Err(err) = ctx.set_function("day_of_month".to_string(),
         Function::new(
             None,
             Box::new(|_args| {
                 Ok(Value::Int(Utc::now().naive_utc().day() as i64))
             }),
         ),
        ) {
        return Err(err)
    }
    if let Err(err) = ctx.set_function("current_ordinal".to_string(),
         Function::new(
             None,
             Box::new(|_args| {
                 Ok(Value::Int(Utc::now().naive_utc().ordinal() as i64))
             }),
         ),
        ) {
        return Err(err)
    }
    if let Err(err) = ctx.set_function("current_weekday".to_string(),
         Function::new(
             None,
             Box::new(|_args| {
                 Ok(Value::String(format!("{:?}", Utc::now().naive_utc().weekday())))
             }),
         ),
        ) {
        return Err(err)
    }
    if let Err(err) = ctx.set_function("current_epoch_secs".to_string(),
         Function::new(
             None,
             Box::new(|_args| {
                 Ok(Value::Int(Utc::now().naive_utc().timestamp()))
             }),
         ),
        ) {
        return Err(err)
    }
    if let Err(err) = ctx.set_function("date_epoch_secs".to_string(),
         Function::new(
             Some(3),
             Box::new(|args| {
                 if let (Value::Int(year), Value::Int(month), Value::Int(day)) = (args[0].clone(), args[1].clone(), args[2].clone()) {
                    let d = NaiveDate::from_ymd(year as i32, month as u32, day as u32).and_hms(0, 0, 0).timestamp();
                    Ok(Value::Int(d))
                 } else {
                     Err(EvalexprError::expected_number(args[0].clone()))
                 }
             }),
         ),
        ) {
        return Err(err)
    }
    if let Err(err) = ctx.set_function("datetime_epoch_secs".to_string(),
         Function::new(
             Some(6),
             Box::new(|args| {
                 if let (Value::Int(year), Value::Int(month), Value::Int(day), Value::Int(hour), Value::Int(min), Value::Int(sec)) = (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone(), args[4].clone(), args[5].clone()) {
                    let d = NaiveDate::from_ymd(year as i32, month as u32, day as u32).and_hms(hour as u32, min as u32, sec as u32).timestamp();
                    Ok(Value::Int(d))
                 } else {
                     Err(EvalexprError::expected_number(args[0].clone()))
                 }
             }),
         ),
        ) {
        return Err(err)
    }
    Ok(true)
}

fn add_context_func(ctx: &mut HashMapContext, name: &str, value: ValueWrapper) -> Result<bool, evalexpr::EvalexprError> { 
    match value {
        ValueWrapper::Bool(b) => ctx.set_value(name.to_string(), Value::Boolean(b))?,
        ValueWrapper::String(ref s) => ctx.set_value(name.to_string(), Value::String(s.clone()))?,
        ValueWrapper::Int(i) => ctx.set_value(name.to_string(), Value::Int(i))?,
        ValueWrapper::Float(f) => ctx.set_value(name.to_string(), Value::Float(f))?,
    }
    Ok(true)
}



#[cfg(test)]
mod tests {
    use plexrbac::utils::evaluator::*;
    use std::collections::HashMap;

    #[test]
    fn test_evaluate() {
        let properties = HashMap::new();
        assert_eq!(Ok(true), evaluate("2 == 2", &properties));
    }
}


