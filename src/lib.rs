use std::{
	collections::BTreeSet,
	rc::Rc,
	sync::{Arc, Mutex},
	time::SystemTime,
};

use custard_macros::{attach_datachunk, attach_task};

use custard_use::{
	composition::loaded::datachunk_getter::DatachunkGetter,
	concurrency::possibly_poisoned_mutex::PossiblyPoisonedMutex,
	errors::task_composition_errors::custard_not_in_cycle_error::CustardNotInCycleError,
	identify::{datachunk_name::FullDatachunkName, task_name::FullTaskName},
	user_types::{
		datachunk::Datachunkable,
		task::{TaskClosureType, Taskable},
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

impl Datachunkable for TestDatachunkA {}

attach_datachunk!(TestDatachunkA);

#[derive(Debug, Deserialize)]
pub struct TestTaskA {
	counter: u32,
	funny_string: String,
	#[serde(default = "SystemTime::now")]
	time: SystemTime,
}

impl Taskable for TestTaskA {
	fn handle_control_flow_update(&mut self, _this_name: &FullTaskName, _other_name: &FullTaskName, _control_flow: &TaskControlFlow) -> bool {
		//any kind of control flow update causes this to quit (not)
		false
	}

	fn run(&mut self, task_name: FullTaskName, datachunk_getter: Arc<DatachunkGetter>) -> TaskClosureType {
		let mut shared_data = if task_name == FullTaskName::new("custard-basics-test-a".to_owned(), "test-task-c".to_owned()) { Some(datachunk_getter.get_mut::<TestDatachunkA>(&FullDatachunkName::new("custard-basics-test-a".to_owned(), "test-datachunk-a".to_owned())).or_panic()) } else { None };

		Box::new(Mutex::new(move |data: Arc<PossiblyPoisonedMutex<dyn Taskable>>| {
			//play with datachunk
			if let Some(v) = &mut shared_data {
				v.field_a = !v.field_a;
				v.field_b *= 2;
				println!("{}, {}, {}", v.field_a, v.field_b, v.field_c);
			}

			let mut object = data.lock();
			let data = object.downcast_mut::<TestTaskA>().unwrap();

			if data.counter == 0 {
				data.time = SystemTime::now();
			}

			data.counter += 1;

			println!("Counter: {}", data.counter);
			println!("Enter your option and press enter:\nContinue (c)\nError(e)\nFullReload(f)\nPartialReload(p)\nStopAll(a)\nStopThis(s)\npanic(!)\n");

			let mut input_string = "".to_owned();

			std::io::stdin().read_line(&mut input_string).unwrap();
			input_string.retain(|c| !c.is_whitespace());

			return match input_string.as_str() {
				"c" => TaskControlFlow::Continue,
				"e" => TaskControlFlow::Err(Rc::new(CustardNotInCycleError { offending_task: task_name.clone() })),
				"f" => TaskControlFlow::FullReload,
				"p" => TaskControlFlow::PartialReload({
					let mut definitely_reload = BTreeSet::new();

					println!("Reload this crate? (y/n)");

					let mut input_string = "".to_owned();

					std::io::stdin().read_line(&mut input_string).unwrap();
					input_string.retain(|c| !c.is_whitespace());
					input_string = input_string.to_lowercase();

					match input_string.as_str() {
						"y" | "yes" | "1" => {
							definitely_reload.insert(task_name.crate_name.clone());
						}
						"n" | "no" | "0" => {}
						_ => {
							println!("Didn't receive valid command (y/yes/1/n/no/0), continuing");
							return TaskControlFlow::Continue;
						}
					}
					Arc::new(definitely_reload)
				}),
				"a" => TaskControlFlow::StopAll,
				"s" => TaskControlFlow::StopThis,
				"!" => panic!("user-triggered panic"),
				_ => {
					println!("Not a command!");
					TaskControlFlow::Continue
				}
			};
		}))
	}
}

impl Drop for TestTaskA {
	fn drop(&mut self) {
		println!("drop works: {}", self.funny_string)
	}
}

attach_task!(TestTaskA);
