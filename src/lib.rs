use std::{
	cell::RefCell,
	sync::{Arc, RwLock},
	time::SystemTime,
};

use custard_macros::{attach_datachunk, attach_task};

use custard_use::{
	errors::tasks_result::TasksResult,
	user_types::{
		datachunk::Datachunk,
		task::{Task, TaskClosureType},
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
pub struct TestTaskA {
	counter: RefCell<u32>,
	funny_string: String,
	#[serde(default = "set_time_default")]
	time: RefCell<SystemTime>,
}

fn set_time_default() -> RefCell<SystemTime> {
	//we're actually going to completely ignore this initial value in order to get a more accurate result, but consider it a tutorial
	RefCell::new(SystemTime::now())
}

impl Task for TestTaskA {
	fn run(self: Arc<Self>) -> TaskClosureType {
		Box::new(move |_value: Arc<RwLock<TasksResult>>| {
			let mut counter = self.counter.borrow_mut();
			if *counter == 0 {
				let mut time = self.time.borrow_mut();
				*time = SystemTime::now();
			}
			*counter += 1;
			// println!("hello from task closure: {}", self.funny_string);

			Ok(())
		})
	}
}

impl Drop for TestTaskA {
	fn drop(&mut self) {
		let time = self.time.borrow();
		let time_since_last = time.elapsed().unwrap();
		println!("time elapsed: {}", time_since_last.as_nanos());
		println!("counter: {}", *self.counter.borrow());
	}
}

attach_task!(TestTaskA);
