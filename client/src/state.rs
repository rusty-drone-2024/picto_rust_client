pub struct Client {
    pub client_data: ClientBackendData,
}

pub struct ClientBackendData;

impl Client {
    pub(crate) fn new(id: u32) -> Self {
        Client {
            client_data: ClientBackendData,
        }
    }
}
