use std::{
	cell::RefCell,
	sync::{Arc, Mutex},
	time::SystemTime,
};

use custard_macros::{attach_datachunk, attach_task};

use custard_use::{
	identify::{custard_name::CustardName, task_name::FullTaskName},
	user_types::{
		datachunk::Datachunk,
		task::{Task, TaskClosureType, TaskData, TaskImpl},
		task_control_flow::task_control_flow::TaskControlFlow,
	},
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TestDatachunkA {
	field_a: bool,
	field_b: u32,
	field_c: String,
}

impl Datachunk for TestDatachunkA {}

attach_datachunk!(TestDatachunkA);

#[derive(Debug, Deserialize)]
pub struct TestTaskAData {
	// #[serde(default = "set_counter_default")]
	counter: Mutex<u32>,
	funny_string: String,
	#[serde(default = "set_time_default")]
	time: Mutex<SystemTime>,
}

impl TaskData for TestTaskAData {}

#[derive(Debug)]
pub struct TestTaskAImpl();

fn set_counter_default() -> Mutex<u32> {
	Mutex::new(0)
}

fn set_time_default() -> Mutex<SystemTime> {
	//we're actually going to completely ignore this initial value in order to get a more accurate result, but consider it a tutorial
	Mutex::new(SystemTime::now())
}

impl TaskImpl for TestTaskAImpl {
	fn handle_control_flow_update(&self, task_data: &dyn TaskData, this_name: &FullTaskName, other_name: &FullTaskName, control_flow: &TaskControlFlow) -> bool {
		true
	}

	fn run(&self, _task_data: &dyn TaskData, task_name: FullTaskName) -> TaskClosureType {
		Box::new(Mutex::new(move |data: Arc<Mutex<dyn TaskData>>| {
			let object = data.lock().unwrap();
			let data = object.downcast_ref::<TestTaskAData>().unwrap();
			let mut counter = data.counter.lock().unwrap();
			let mut time = data.time.lock().unwrap();

			if *counter == 0 {
				*time = SystemTime::now();
			}

			*counter += 1;

			if *counter == 10_000 && task_name.task_name.get() == "test-task-a" {
				let time_since_last = time.elapsed().unwrap();

				println!("time elapsed: {}", time_since_last.as_nanos());
				println!("counter: {}", counter);

				return TaskControlFlow::Reload;
			}

			TaskControlFlow::Continue
			// println!("hello from task closure: {}", self.funny_string);
		}))
	}
}

impl Drop for TestTaskAData {
	fn drop(&mut self) {
		println!("drop works: {}", self.funny_string)
	}
}

attach_task!["TestTaskAImpl:TestTaskAData"];
