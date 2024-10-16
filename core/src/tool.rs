use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub struct FunctionToolError(String);

impl std::fmt::Display for FunctionToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FunctionToolError {}

pub fn parse_i32(arg_value: &str) -> std::result::Result<i32, Box<dyn Error>> {
    arg_value.parse::<i32>().map_err(|e| {
        Box::new(FunctionToolError(format!(
            "Expected i32 for argument: {}",
            e
        ))) as Box<dyn Error>
    })
}

pub fn parse_f32(arg_value: &str) -> std::result::Result<f32, Box<dyn Error>> {
    arg_value.parse::<f32>().map_err(|e| {
        Box::new(FunctionToolError(format!(
            "Expected f32 for argument: {}",
            e
        ))) as Box<dyn Error>
    })
}

pub fn parse_bool(arg_value: &str) -> std::result::Result<bool, Box<dyn Error>> {
    match arg_value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => {
            Err(Box::new(FunctionToolError("Expected bool for argument".into())) as Box<dyn Error>)
        }
    }
}

pub fn parse_string(arg_value: &str) -> std::result::Result<String, Box<dyn Error>> {
    Ok(arg_value.to_string())
}

#[derive(Debug)]
pub enum SupportedType {
    I32(i32),
    F32(f32),
    Bool(bool),
    String(String),
}

pub fn parse_argument(
    arg_type: &str,
    arg_value: &str,
) -> std::result::Result<SupportedType, Box<dyn Error>> {
    match arg_type {
        "i32" => parse_i32(arg_value).map(SupportedType::I32),
        "f32" => parse_f32(arg_value).map(SupportedType::F32),
        "bool" => parse_bool(arg_value).map(SupportedType::Bool),
        "String" => parse_string(arg_value).map(SupportedType::String),
        _ => Err(Box::new(FunctionToolError("Invalid type".into()))),
    }
}

pub fn get_parsers() -> HashMap<
    &'static str,
    fn(&SupportedType) -> std::result::Result<Box<dyn std::any::Any>, Box<dyn Error>>,
> {
    let mut parsers: HashMap<
        &str,
        fn(&SupportedType) -> std::result::Result<Box<dyn std::any::Any>, Box<dyn Error>>,
    > = HashMap::new();
    parsers.insert("i32", |v| {
        if let SupportedType::I32(val) = v {
            Ok(Box::new(*val))
        } else {
            Err(Box::new(FunctionToolError("Type mismatch for i32".into())))
        }
    });
    parsers.insert("f32", |v| {
        if let SupportedType::F32(val) = v {
            Ok(Box::new(*val))
        } else {
            Err(Box::new(FunctionToolError("Type mismatch for f32".into())))
        }
    });
    parsers.insert("bool", |v| {
        if let SupportedType::Bool(val) = v {
            Ok(Box::new(*val))
        } else {
            Err(Box::new(FunctionToolError("Type mismatch for bool".into())))
        }
    });
    parsers.insert("String", |v| {
        if let SupportedType::String(val) = v {
            Ok(Box::new(val.clone()))
        } else {
            Err(Box::new(FunctionToolError(
                "Type mismatch for String".into(),
            )))
        }
    });
    parsers
}

pub struct FunctionTool {
    name: String,
    function:
        Box<dyn Fn(&[SupportedType]) -> std::result::Result<String, Box<dyn Error>> + Send + Sync>,
    arg_names: Vec<String>,
    arg_types: Vec<String>,
    return_type: String,
}

impl FunctionTool {
    pub fn run(&self, arguments_w_val: Value) -> std::result::Result<String, Box<dyn Error>> {
        let arguments = arguments_w_val["arguments"].as_array().ok_or_else(|| {
            Box::new(FunctionToolError("Invalid arguments format".into())) as Box<dyn Error>
        })?;
        let mut ordered_vals = Vec::new();

        for (i, arg_name) in self.arg_names.iter().enumerate() {
            let arg_value = arguments.iter().find_map(|arg| {
                let obj = arg.as_object().unwrap();
                obj.get(arg_name)
            });

            if let Some(arg_value) = arg_value {
                let arg_str = arg_value.as_str().ok_or_else(|| {
                    Box::new(FunctionToolError("Invalid argument value".into())) as Box<dyn Error>
                })?;
                let parsed_arg = parse_argument(&self.arg_types[i], arg_str)?;
                ordered_vals.push(parsed_arg);
            } else {
                return Err(Box::new(FunctionToolError(format!(
                    "Missing argument: {}",
                    arg_name
                ))));
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
        let return_type = stringify!($ret_type).to_string();

        let func = Box::new(move |args: &[SupportedType]| -> Result<String, Box<dyn Error>> {
            let parsers = get_parsers();

            let mut iter = args.iter();
            $(
                let $arg_name = {
                    let arg = iter.next().ok_or_else(|| Box::<dyn Error>::from("Insufficient arguments"))?;
                    let parser = parsers.get(stringify!($arg_type))
                        .ok_or_else(|| Box::<dyn Error>::from(format!("Parser not found for type: {}", stringify!($arg_type))))?;
                    let parsed_value = parser(arg)?;
                    *parsed_value.downcast::<$arg_type>()
                        .map_err(|_| Box::<dyn Error>::from("Type downcast failed"))?
                };
            )*

            $func_name($($arg_name),*).map_err(|e| Box::<dyn Error>::from(e))
        }) as Box<dyn Fn(&[SupportedType]) -> Result<String, Box<dyn Error>> + Send + Sync>;

        FunctionTool {
            name: serde_json::from_str::<Value>($json_description).unwrap()["name"]
                .as_str()
                .unwrap()
                .to_string(),
            function: func,
            arg_names,
            arg_types,
            return_type
        }
    }};
}

pub struct FunctionCallInput {
    pub arguments_obj: Value,
    pub function_name: String,
    pub return_type: String,
}

// pub struct FunctionCall {
//     pub id: String,
//     pub args: &'static [u8],
//     pub name: String,
// }

// impl FunctionCall {
//     pub fn run(self) {
//         let bindings = &STORE;
//         let function = bindings.get(&self.name).unwrap();

//         function(self.args);
//     }
// }


pub fn get_current_weather(location: String, unit: String) -> Result<String, String> {
    if (location.as_str() == "New York") {
        Ok(format!("Weather for {} in {}", location, unit))
    } else {
        Err("I only forecast New York".to_string())
    }
}

pub fn process_values(a: i32, b: f32, c: bool, d: String, e: i32) -> String {
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
               { "location": "New York"},
                {"unit": "celsius"}
            ]
        });

        let function_tool = create_tool_with_function!(fn get_current_weather(location: String, unit: String) -> Result<String, String>, weather_tool_json);
        println!("tool name: {:?}", function_tool.name);
        println!("tool arg names: {:?}", function_tool.arg_names);
        println!("tool arg types: {:?}", function_tool.arg_types);
        println!("tool return type: {:?}", function_tool.return_type);

        let result = function_tool
            .run(llm_output)
            .expect("Function run failed");
        assert_eq!(result, "Weather for Glasgow, Scotland in celsius");
    }
}
