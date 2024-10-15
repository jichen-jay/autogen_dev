use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use once_cell::sync::Lazy;
use crate::common_types::*;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn  parse_i32(arg_value: &str) -> i32 {
    arg_value.parse::<i32>().expect("Expected i32 for argument")
}

pub fn  parse_f32(arg_value: &str) -> f32 {
    arg_value.parse::<f32>().expect("Expected f32 for argument")
}

pub fn  parse_bool(arg_value: &str) -> bool {
    match arg_value {
        "true" => true,
        "false" => false,
        _ => panic!("Expected bool for argument"),
    }
}

pub fn  parse_string(arg_value: &str) -> String {
    arg_value.to_string()
}

#[derive(Debug)]
pub enum SupportedType {
    I32(i32),
    F32(f32),
    Bool(bool),
    String(String),
}

pub fn  parse_argument(arg_type: &str, arg_value: &str) -> SupportedType {
    match arg_type {
        "i32" => SupportedType::I32(parse_i32(arg_value)),
        "f32" => SupportedType::F32(parse_f32(arg_value)),
        "bool" => SupportedType::Bool(parse_bool(arg_value)),
        "String" => SupportedType::String(parse_string(arg_value)),
        _ => panic!("Invalid type"),
    }
}

pub fn  get_parsers() -> HashMap<&'static str, fn(&SupportedType) -> Box<dyn std::any::Any>> {
    let mut parsers: HashMap<&str, fn(&SupportedType) -> Box<dyn std::any::Any>> = HashMap::new();
    parsers.insert("i32", |v| {
        if let SupportedType::I32(val) = v {
            Box::new(*val)
        } else {
            panic!("Type mismatch")
        }
    });
    parsers.insert("f32", |v| {
        if let SupportedType::F32(val) = v {
            Box::new(*val)
        } else {
            panic!("Type mismatch")
        }
    });
    parsers.insert("bool", |v| {
        if let SupportedType::Bool(val) = v {
            Box::new(*val)
        } else {
            panic!("Type mismatch")
        }
    });
    parsers.insert("String", |v| {
        if let SupportedType::String(val) = v {
            Box::new(val.clone())
        } else {
            panic!("Type mismatch")
        }
    });
    parsers
}

pub struct FunctionTool {
    name: String,
    function: Box<dyn Fn(&[SupportedType]) -> Result<String> + Send + Sync>,
    arg_names: Vec<String>,
    arg_types: Vec<String>,
}

impl FunctionTool {
    pub fn  call(&self, arguments_w_val: Value) -> Result<String> {
        let arguments = arguments_w_val["arguments"]
            .as_array()
            .ok_or("Invalid arguments format")?;
        let mut ordered_vals = Vec::new();

        for (i, arg_name) in self.arg_names.iter().enumerate() {
            let arg_value = arguments.iter().find_map(|arg| {
                let obj = arg.as_object().unwrap();
                obj.get(arg_name)
            });

            if let Some(arg_value) = arg_value {
                let arg_str = arg_value.as_str().ok_or("Invalid argument value")?;
                let parsed_arg = parse_argument(&self.arg_types[i], arg_str);
                ordered_vals.push(parsed_arg);
            } else {
                return Err(format!("Missing argument: {}", arg_name).into());
            }
        }

        (self.function)(&ordered_vals)
    }
}

// Macro to create a Tool with a function and argument parsing logic
#[macro_export]
macro_rules! create_tool_with_function {
    (
        fn $func_name:ident($($arg_name:ident : $arg_type:ty),*) -> $ret_type:ty,
        $json_description:expr
    ) => {{
        let arg_names = vec![$(stringify!($arg_name).to_string()),*];
        let arg_types = vec![$(stringify!($arg_type).to_string()),*];

        let func = Box::new(move |args: &[SupportedType]| -> Result<String> {
            let parsers = get_parsers();

            let mut iter = args.iter();
            $(
                let $arg_name = {
                    let arg = iter.next().unwrap();
                    let parser = parsers.get(stringify!($arg_type)).expect("Parser not found");
                    *parser(arg).downcast::<$arg_type>().expect("Type mismatch")
                };
            )*

            let result = $func_name($($arg_name),*);
            Ok(result)
        }) as Box<dyn Fn(&[SupportedType]) -> Result<String> + Send + Sync>;

        FunctionTool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names: arg_names,
            arg_types: arg_types,
        }
    }};
}

pub fn  get_current_weather(location: String, unit: String) -> String {
    format!("Weather for {} in {}", location, unit)
}

pub fn  process_values(a: i32, b: f32, c: bool, d: String, e: i32) -> String {
    format!(
        "Processed: a = {}, b = {}, c = {}, d = {}, e = {}",
        a, b, c, d, e
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_process_values() {
        let json_description = r#"
        {
            "name": "process_values",
            "description": "Processes up to 5 different types of values",
            "parameters": {
                "type": "object",
                "properties": {
                    "a": {
                        "type": "i32",
                        "description": "An integer value"
                    },
                    "b": {
                        "type": "f32",
                        "description": "A floating-point value"
                    },
                    "c": {
                        "type": "bool",
                        "description": "A boolean value"
                    },
                    "d": {
                        "type": "string",
                        "description": "A string value"
                    },
                    "e": {
                        "type": "i32",
                        "description": "Another integer value"
                    }
                },
                "required": ["a", "b", "c", "d", "e"]
            }
        }
        "#;

        let json_input = json!({
            "arguments": [
                { "a": "42" },
                { "b": "3.14" },
                { "c": "true" },
                { "d": "example" },
                { "e": "100" }
            ]
        });

        let function_tool = create_tool_with_function!(fn process_values(a: i32, b: f32, c: bool, d: String, e: i32) -> String, json_description);
        let result = function_tool.call(json_input).expect("Tool call failed");

        assert_eq!(
            result,
            "Processed: a = 42, b = 3.14, c = true, d = example, e = 100"
        );
    }

    #[test]
    fn test_get_current_weather() {
        let weather_tool_json = r#"
        {
            "name": "get_current_weather",
            "description": "Get the current weather in a given location",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"],
                        "description": "The unit of measurement"
                    }
                },
                "required": ["location", "unit"]
            }
        }
        "#;

        let llm_output = json!({
            "arguments": [
               { "location": "Glasgow, Scotland"},
                {"unit": "celsius"}
            ]
        });

        let function_tool = create_tool_with_function!(fn get_current_weather(location: String, unit: String) -> String, weather_tool_json);
        let result = function_tool.call(llm_output).expect("Function call failed");
        assert_eq!(result, "Weather for Glasgow, Scotland in celsius");
    }
}
