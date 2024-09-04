use crate::pxolly::dispatch::handler::Handler;
use crate::pxolly::types::events::event_type::EventType;
use crate::pxolly::types::responses::errors::{PxollyErrorType, PxollyWebhookError};
use crate::pxolly::types::responses::webhook::PxollyWebhookResponse;
use crate::vkontakte::api::VKontakteAPI;
use crate::vkontakte::types::categories::Categories;
use crate::vkontakte::types::params::execute::ExecuteParams;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct InviteUserObject {
    chat_id: String,
    chat_local_id: Option<u64>,
    user_id: u64,
    is_expired: Option<u8>,
    visible_messages_count: Option<u8>, // В доке такого нет, но раньше точно было. Поэтому так
}

pub struct InviteUser {
    pub(crate) vkontakte: VKontakteAPI,
}

impl Handler for InviteUser {
    const EVENT_TYPE: EventType = EventType::InviteUser;
    type EventObject = InviteUserObject;

    async fn handle(
        &self,
        object: Self::EventObject,
    ) -> Result<PxollyWebhookResponse, PxollyWebhookError> {
        let params = serde_json::json!({
            "visible_messages_count": object.visible_messages_count.unwrap_or(0),
            "member_id": object.user_id,
            "chat_id": object.chat_local_id.ok_or_else(PxollyWebhookError::chat_not_found)?,
        });
        match self
            .vkontakte
            .execute::<i64>(ExecuteParams {
                code: EXECUTE_INVITE_CODE.into(),
                extras: params,
            })
            .await?
        {
            -100 => Err(PxollyWebhookError {
                message: None,
                error_type: PxollyErrorType::NotInFriends,
            }),
            _ => Ok(PxollyWebhookResponse::new(true)),
        }
    }
}

const EXECUTE_INVITE_CODE: &str = r#"
if(API.friends.areFriends({user_ids:Args.member_id})[0].friend_status==3) {
    return API.messages.addChatUser({
        chat_id: Args.chat_id,
        user_id: Args.member_id,
        visible_messages_count: Args.visible_messages_count
    });
}
return -100;
"#;
