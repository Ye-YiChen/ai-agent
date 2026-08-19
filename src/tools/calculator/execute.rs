use schemars::JsonSchema;

pub fn calculator(
    operator: &str,
    first_number: f64,
    second_number: f64,
) -> anyhow::Result<f64, String> {
    match operator {
        "add" => Ok(first_number + second_number),
        "subtract" => Ok(first_number - second_number),
        "multiply" => Ok(first_number * second_number),
        "divide" => {
            if second_number == 0.0 {
                Err("Division by zero is not allowed.".to_string())
            } else {
                Ok(first_number / second_number)
            }
        }
        _ => Err(format!("Unsupported operator: {}", operator)),
    }
}
#[derive(Debug, serde::Deserialize, JsonSchema)]
pub struct  CalculatorArgs {
    pub operator: String,
    pub first_number: f64,
    pub second_number: f64,
}