use serde_json::Value;

pub struct Tool {
    pub name: String,
    pub description: String,
    pub tool_meta: Value,
    pub parameters: Value,

}


// {
//     "name": "get_current_weather",
//     "description": "Get the current weather in a given location",
//     "parameters": {
//     "properties": {
//     "location": {
//     "description": "The city and state, e.g. San Francisco, CA",
//     "type": "string"
//     },
//     "unit": {
//     "enum": [
//     "celsius",
//     "fahrenheit"
//     ],
//     "type": "string"
//     }
//     },
//     "required": [
//     "location"
//     ],
//     "type": "object"
//     }
//     }