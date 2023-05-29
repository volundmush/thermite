use mlua::{Lua, Compiler, LuaOptions, Function, Chunk, Thread, VmState};
use std::sync::{Arc, RwLock};
use regex::bytes::Regex;
use once_cell::sync::Lazy;
use std::time::{Duration, Instant};
use mlua::Error as LuaError;
use tokio_stream::wrappers::IntervalStream;
use tokio::time;
use futures::{
    sink::{SinkExt},
    stream::{StreamExt}
};

static COMPILER: Lazy<mlua::Compiler> = Lazy::new(|| mlua::Compiler::new());

#[derive(Debug, Clone)]
pub enum CompileResults {
    Compiled(Vec<u8>),
    Error(String)
}

#[derive(Debug, Clone)]
pub struct LuaCode {
    pub code: String,
    pub compiled_code: Option<CompileResults>,
}

impl LuaCode {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
            compiled_code: None,
        }
    }

    pub fn compile(&mut self) {
        let result: Vec<u8> = COMPILER.compile(&self.code);

        let re = Regex::new(r"^:\d+: ").unwrap();

        self.compiled_code = if re.is_match(&result) {
            Some(CompileResults::Error(String::from_utf8_lossy(&result).to_string()))
        } else {
            Some(CompileResults::Compiled(result))
        };
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum TaskState {
    Loading,
    Ready,
    Running,
    Waiting,
    Interrupted,
    Finished,
    Error,
}

#[derive(Clone, Debug)]
pub struct TaskStatus {
    pub state: TaskState,
    pub error: Option<String>,
    pub start_time: Instant,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self {
            state: TaskState::Loading,
            error: None,
            start_time: Instant::now(),
        }
    }
}

#[derive(Debug)]
pub struct Task {
    pub status: Arc<RwLock<TaskStatus>>,
    interrupted: Arc<std::sync::atomic::AtomicBool>,
    name: String,
    bytecode: Vec<u8>
}

impl Task {
    pub fn new(name: &str, bytecode: Vec<u8>) -> Self {
        Self {
            status: Arc::new(RwLock::new(TaskStatus::default())),
            interrupted: Default::default(),
            name: name.to_string(),
            bytecode
        }
    }

    pub async fn run(&mut self) {
        let mut lua = Lua::new();

        let _ = self.run_inner(&mut lua).await;

    }

    async fn run_inner(&mut self, lua: &mut mlua::Lua) {
        let mut t = {
            let code = lua.load(&self.bytecode).set_name(&self.name);
            let func = match code.into_function() {
                Ok(f) => f,
                Err(e) => {
                    self.status.write().unwrap().state = TaskState::Error;
                    self.status.write().unwrap().error = Some(e.to_string());
                    return;
                },
            };
            let thread = match lua.create_thread(func) {
                Ok(t) => t,
                Err(e) => {
                    self.status.write().unwrap().state = TaskState::Error;
                    self.status.write().unwrap().error = Some(e.to_string());
                    return;
                },
            };
            thread.into_async::<_, u32>(())
        };

        let mut interval_timer = IntervalStream::new(time::interval(Duration::from_millis(1000 * 20)));
        let interrupted = Arc::clone(&self.interrupted); // Clone the interrupted flag

        lua.set_interrupt(move |_| {
            if interrupted.load(std::sync::atomic::Ordering::SeqCst) {
                Ok(mlua::VmState::Yield)
            } else {
                Ok(mlua::VmState::Continue)
            }
        });

        loop {
            let start_watch = Instant::now();


            tokio::select! {
                _ = interval_timer.next() => {
                    self.interrupted.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                l_msg = t.next() => {
                    match l_msg {
                        Some(result) => {
                            match result {
                                Ok(output) => {
                                    // Coroutine has finished
                                    self.status.write().unwrap().state = TaskState::Finished;
                                    break;
                                }
                                Err(e) => match e {
                                    mlua::Error::CoroutineInactive => {
                                        // Check if the coroutine was interrupted
                                        if self.interrupted.load(std::sync::atomic::Ordering::SeqCst) {
                                            self.status.write().unwrap().state = TaskState::Interrupted;
                                            self.interrupted.store(false, std::sync::atomic::Ordering::SeqCst);
                                            tokio::task::yield_now().await;
                                        } else {
                                            // Coroutine has yielded
                                            self.status.write().unwrap().state = TaskState::Waiting;
                                            // We should actually implement a means for it to wait
                                            // for x amount of time...
                                            tokio::task::yield_now().await;
                                        }
                                    }
                                    _ => {
                                        // Coroutine has encountered an error
                                        self.status.write().unwrap().state = TaskState::Error;
                                        self.status.write().unwrap().error = Some(e.to_string());
                                        break;
                                    }
                                }
                            }
                        },
                        None => {

                        }
                    }
                }
            }
        };

    }
}

