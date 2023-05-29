use std::error::Error;
use thermite_server::game::{Task, LuaCode, CompileResults, TaskState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("starting program...");

    let lua_code = r#"
        counter = 0
        function run()
            while true do
                _G.counter = _G.counter + 1
            end
        end
    "#;

    let mut code = LuaCode::new(lua_code);
    code.compile();

    let mut bytecode = Vec::new();
    
    match &code.compiled_code {
        Some(CompileResults::Error(e)) => {
            eprintln!("Lua Compilation error: {}", e);
            std::process::exit(1);
        },
        Some(CompileResults::Compiled(b)) => {
            bytecode = b.clone();
        },
        _ => {}
    }



    let handle = tokio::spawn(async move {
        let mut task = Task::new("test_script", bytecode);
        task.run().await;
    });

    let _ = handle.await;

    Ok(())
}