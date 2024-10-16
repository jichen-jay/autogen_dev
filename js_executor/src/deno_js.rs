use rustyscript::{json_args, Error, Module, Runtime, RuntimeOptions, Undefined};
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_execution() -> Result<(), Error> {
//locad multiple module, solve dependency
/*         let lodash = Module::new("lodash.js", "// lodash implementation");
        let main_module = Module::new("main.js", "// Your main code");

        runtime.load_module(&lodash)?;
        runtime.load_module(&main_module)?; */

        let module = Module::new(
            "test.js",
            "
            let internalValue = 0;
            export const load = (value) => internalValue = value;
            export const getValue = () => internalValue;
            ",
        );

        let mut runtime = Runtime::new(RuntimeOptions {
            timeout: Duration::from_millis(50),
            default_entrypoint: Some("load".to_string()),
            ..Default::default()
        })?;

        let module_handle = runtime.load_module(&module)?;
        runtime.call_entrypoint::<Undefined>(&module_handle, json_args!(2))?;

        let internal_value: i64 =
            runtime.call_function(Some(&module_handle), "getValue", json_args!())?;

        println!("internal_value: {internal_value}");

        Ok(())
    }
}
