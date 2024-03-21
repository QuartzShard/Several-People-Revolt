use futures_util::StreamExt;
use core::fmt;
use std::env;
use tracing::info;
use rive_models;
use rive_http;
use rive_gateway;

#[derive(Debug)]
pub struct Bot {
	http: rive_http::Client,
	gateway: rive_gateway::Gateway,
	bot_user: rive_models::user::User,
}


impl Bot {
	pub async fn new(token: String) -> Result<Bot, BotError> {
		let auth = rive_models::authentication::Authentication::BotToken(token);
		let http = rive_http::Client::new(auth.clone());
		let gateway = rive_gateway::Gateway::connect(auth).await?;
		let bot_user = http.fetch_self().await?;
		http.edit_user(rive_models::data::EditUserData{
			avatar: None,
			profile: None,
			remove: None,
			status: Some(rive_models::user::UserStatus{
				presence: Some(rive_models::user::Presence::Invisible),
				text: None
			})
		}).await?;
		info!("Bot init success!");	
		Ok(Bot{
			http,
			gateway,
			bot_user,
		})
	}
	pub async fn next_event(&mut self) -> Result<rive_models::event::ServerEvent, BotError> {
		match self.gateway.next().await {
			Some(res) => Ok(res?),
			None => Err(BotError::APIError)
		}
	}
	pub async fn send_message(&self, channel: &str, message: String) -> Result<rive_models::message::Message, BotError> {
		info!("Sending message to channel {}", channel);
		let embed = rive_models::embed::SendableEmbed{
			description: Some(message),
			colour: Some("#1d9bf0".to_string()),
			..Default::default()
		};
		let data: rive_models::data::SendMessageData = rive_models::data::SendMessageData {
			embeds: Some(vec![embed]),
			..rive_models::data::SendMessageData::default()
		};
		Ok(self.http.send_message(channel, data).await?)
	}
	pub async fn start_typing(&self, channel: &str) -> Result<(), BotError> {
	   Ok(self.gateway.send(rive_models::event::ClientEvent::BeginTyping { channel: channel.to_string() }).await?)
	}
	pub fn get_bot_user(&self) -> &rive_models::user::User {
		&self.bot_user
	}
}

#[derive(Debug)]
pub enum BotError {
	APIError,
	MissingToken,
}
impl fmt::Display for BotError {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	   write!(f, "Bot Error: {:?}", self)
   } 
}
impl std::error::Error for BotError {
   fn description(&self) -> &str {
	   "An Error in the operation of the bot"
   } 
}
impl From<env::VarError> for BotError {
	fn from(_value: env::VarError) -> Self {
		BotError::MissingToken
	}
}
impl From<rive_http::Error> for BotError {
	fn from(_value: rive_http::Error) -> Self {
		BotError::APIError
	}
}
impl From<rive_gateway::Error> for BotError {
	fn from(_value: rive_gateway::Error) -> Self {
		BotError::APIError
	}
}
