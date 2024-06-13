use std::time::Duration;
use tokio::sync::watch;
use tokio::time;

type Sender = watch::Sender<interfaces::Result<Vec<interfaces::Interface>>>;
type Receiver = watch::Receiver<interfaces::Result<Vec<interfaces::Interface>>>;

#[derive(Debug)]
pub enum InterfacesError {
    WorkerStopped,
    InterfacesError,
}

impl From<watch::error::RecvError> for InterfacesError {
    fn from(_: watch::error::RecvError) -> Self {
        InterfacesError::WorkerStopped
    }
}

pub struct InterfacesClient {
    interfaces: Vec<String>,
    receiver: Receiver,
}

impl InterfacesClient {
    pub fn get(&mut self) -> Result<&Vec<String>, InterfacesError> {
        // This function will be called a lot from the main render loop, so lets try to not copy a
        // bunch of strings around at 60fps by only updating local state when needed.
        if self.receiver.has_changed()? {
            if let Ok(interfaces) = (*self.receiver.borrow_and_update()).as_deref() {
                self.interfaces = interfaces.iter().map(|i| i.name.to_owned()).collect();
            } else {
                return Err(InterfacesError::InterfacesError);
            }
        }

        Ok(&self.interfaces)
    }
}

/// Runs a backgound task that polls the system for CAN interfaces.
pub struct InterfacesTask {
    sender: Sender,
}

impl InterfacesTask {
    pub async fn run(&self) {
        tracing::info!("polling can interfaces");
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            let interfaces = interfaces::Interface::get_all().map(|mut interfaces| {
                interfaces.sort_by(|a, b| a.name.cmp(&b.name));
                interfaces
                    .into_iter()
                    .filter(|i| i.name.contains("can"))
                    .collect()
            });

            tracing::info!("found interfaces: {:?}", interfaces);

            match self.sender.send(interfaces) {
                Ok(()) => interval.tick().await,

                Err(_) => {
                    tracing::info!("ending interfaces task because channel closed");
                    break;
                }
            };
        }
    }
}

pub fn task() -> (InterfacesClient, InterfacesTask) {
    let (sender, receiver) = watch::channel(Ok(vec![]));

    let client = InterfacesClient {
        interfaces: vec![],
        receiver,
    };

    let task = InterfacesTask { sender };

    (client, task)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn client_returns_err_when_task_died() {
        let (mut client, task) = super::task();

        drop(task);

        assert!(client.get().is_err());
    }

    #[tokio::test]
    async fn task_ends_when_client_is_dropped() {
        let (client, task) = super::task();

        let handle = tokio::spawn(async move { task.run().await });

        drop(client);

        assert!(handle.await.is_ok());
    }
}
