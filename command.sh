 //tests in submodule
 cargo test -p python_executor --lib -- vanilla_python::tests::test_run_python_script   --nocapture

//tests in root
 cargo test -p js_executor --lib -- tests::test_javascript   --nocapture