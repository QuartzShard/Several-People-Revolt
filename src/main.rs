mod bot;
use bot::Bot;
use rive_models::event::ServerEvent;
use tracing::{error, info, debug};
use std::env;
use std::fmt;
use tracing_subscriber;
use rive_models::*;


#[cfg(debug_assertions)]
fn logger_init() {
	tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();
}
#[cfg(not(debug_assertions))]
fn logger_init() {
	tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
}

#[tokio::main()]
async fn main() -> Result<(), bot::BotError> {

	logger_init();
	info!("Starting...");

	let mut typer_bot: Bot = Bot::new(env::var("TOKEN")?).await?;
	let mut counter: u64 = 1;
	// Main program loop. Wait for events, respond appropriately
	loop {
        if counter > 64 {
            error!("Cannot get next event from gateway");
			info!("Restarting bot");
            typer_bot = Bot::new(env::var("TOKEN")?).await?;
        };
        match typer_bot.next_event().await {
            Ok(event) => match handle_event(event, &typer_bot).await {
				Ok(()) => counter = 1,
				Err(error) => {
                    error!("Error handling event: {}", error); 
                    tokio::time::sleep(tokio::time::Duration::from_secs(counter)).await;
                    counter += counter;
                }
			},
            Err(error) => {
                error!("Error getting next event: {}", error);
                tokio::time::sleep(tokio::time::Duration::from_secs(counter)).await;
                counter += counter;
            }
        };
    }
}

// Typing shorthand, again
type EventResult = Result<(), EventHandleError>;

// Could've been an inline, but this is prettier
async fn handle_event(event: ServerEvent, typer_bot: &Bot) -> EventResult {
	// Log any and all event if we're a debug build
	debug!("Got event: {:?}", event);
	match event {
		ServerEvent::Ready(_) => {info!("Bot Ready"); Ok(())},
		ServerEvent::Message(ev) => handle_message(ev, typer_bot).await,
		// If it's not one of the event listed above, we don't care. Just return Ok(())
		// as if we handled it. (Which we did, by ignoring it :trl:)
		_ => Ok(()),
	}
}

async fn handle_message(msg: message::Message, typer_bot: &Bot) -> EventResult { 
	if msg.author == typer_bot.get_bot_user().id{
		debug!("Ignoring own message");
		return Ok(());
	}; 
	debug!("Got a message!");
    if msg.content.unwrap_or("".to_string()).contains(&("<@".to_string() + &typer_bot.get_bot_user().id + ">")) {
        typer_bot.start_typing(&msg.channel).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        typer_bot.send_message(&msg.channel, get_eggman_message(&msg.author, typer_bot)).await?;
        Ok(())
    } else {
        Ok(typer_bot.start_typing(&msg.channel).await?)
    } 
}

fn get_eggman_message(target: &str, typer_bot: &Bot) -> String {
    format!("## @<@{bot_id}> tweeted:\n# :01HS7KZTA0HXX72GPS1D89F2JV:\nI've come to make an announcement: **<@{id}>'s** a ***bitch-ass motherfucker***. They pinged my **fucking** bot. That's right, they took their **hedgehog** fuckin' **quilly** ***ping*** out and they ***pinged my FUCKING bot***, and they said their ping was ***\"THIS BIG\"***, and I said \"That's disgusting!\" So I'm making a callout post on my Twitter.com: **<@{id}>, you got a** ***small*** **ping!** It's the size of this walnut except ***WAY smaller!*** And guess what? ***Here's what my pong looks like!*** That's right, baby. *All* points, *no* quills, *no* pillows, look at that, it looks like ***two balls and a bong.*** They fucked my bot, so *guess what?* **I'm gonna fuck the earth!** That's right, this is what you get! My ***SUPER LASER PISS!*** Except I'm not gonna piss on the *Earth*, I'm gonna go **higher**. I'm *pissing* on the ***MOOOON!***\n\n**HOW DO YOU LIKE THAT OBAMA?**\n**I PISSED ON THE MOON, YOU** ***IDIOT!***\n\nYou have twenty-three hours before the piss *DRRRROPLLLETS* hit the ***fucking*** *Earth*, now get out of my fucking sight, before I piss on you too!", bot_id=typer_bot.get_bot_user().id, id=target)
}
// Where did we go wrong?
#[derive(Debug)]
enum EventHandleError {
	Message,
}
impl fmt::Display for EventHandleError {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       write!(f, "Event Error: {:?}", self)
   } 
}
//impl std::error::Error for EventHandleError {
   //fn description(&self) -> &str {
   //   "An Error in event handling"
   //} 
//}
impl From<bot::BotError> for EventHandleError {
   fn from(value: bot::BotError) -> Self {
		match value {
			bot::BotError::APIError => Self::Message,
			bot::BotError::MissingToken => panic!("Lost token"),
		}
   } 
}
