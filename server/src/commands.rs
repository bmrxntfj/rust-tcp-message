use bytes::Bytes;
use lib::message::{Message,BusinessRequest, BusinessResponse};

pub fn process_client_request(message: Message) -> anyhow::Result<Message> {
    let req = serde_json::from_slice::<BusinessRequest>(message.body.as_ref())?;
    log::trace!("read->{},{:?}", message.id, req);
    let ticket_res = serde_json::to_string(&BusinessResponse {
        order_no: req.order_no,
        status: "CREATED".to_owned(),
    })?;
    log::trace!("write->{},{:?}", message.id, ticket_res);
    Ok(Message::new(
        message.id,
        Bytes::copy_from_slice(ticket_res.as_bytes()),
    ))
}
