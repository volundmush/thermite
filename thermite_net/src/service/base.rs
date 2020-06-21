use tokio::prelude::*;
use tokio::sync::mspc;

pub struct TaskHolder<CT, CF> {
    pub name: String,
    pub channel_to_task: mspc::Sender<CT>,
    pub channel_from_task: mspc::Receiver<CF>,
    pub handle: tokio::task::JoinHandle<_>

}

pub struct Task<CT, CF, ST> {
    pub name: String,
    pub channel_to_holder: mspc::Sender<CF>,
    pub channel_from_holder: mspc::Receiver<CT>,
    pub state: ST
}

impl Task<CT, CF, ST> {
    async fn run(&mut self) {

    }
}

impl TaskHolder<CT, CF> {
    async fn new<ST>(name: String) -> TaskHolder<CT, CF> {

        let (channel_to_task, channel_from_holder) = mspc::channel(10);
        let (channel_to_holder, channel_from_task) = mspc::channel(10);

        let new_task = Task {
            name: name.clone(),
            channel_to_holder,
            channel_from_holder,
            state
        };

        let handle = tokio::spawn(async move {
            let task = new_task;
            task.run().await;
        });

        TaskHolder {
            name: name.clone(),
            channel_to_task,
            channel_from_task,
            handle
        }

    }
}