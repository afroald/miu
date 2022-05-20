use interfaces;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct CanInterfaces {
    interfaces: Arc<Mutex<Result<Vec<String>, interfaces::InterfacesError>>>,
}

impl CanInterfaces {
    pub fn new() -> Self {
        let instance = Self {
            interfaces: Arc::new(Mutex::new(Ok(vec![]))),
        };

        let available_interfaces = Arc::clone(&instance.interfaces);
        thread::spawn(move || loop {
            match interfaces::Interface::get_all() {
                Ok(detected_interfaces) => {
                    let mut detected_interface_names: Vec<String> = detected_interfaces
                        .into_iter()
                        .map(|interface| interface.name.clone())
                        .filter(|interface_name| interface_name.contains("can"))
                        .collect();
                    detected_interface_names.sort();

                    let mut available_interfaces = available_interfaces.lock().unwrap();
                    *available_interfaces = Ok(detected_interface_names);
                }
                Err(error) => {
                    let mut available_interfaces = available_interfaces.lock().unwrap();
                    *available_interfaces = Err(error);
                }
            }
            thread::sleep(Duration::from_secs(1));
        });

        instance
    }
}

impl Deref for CanInterfaces {
    type Target = Arc<Mutex<Result<Vec<String>, interfaces::InterfacesError>>>;

    fn deref(&self) -> &Self::Target {
        &self.interfaces
    }
}
