use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write;
use std::sync::Arc;

use serenity::async_trait;
use serenity::client::bridge::gateway::{ShardId, ShardManager};
use serenity::framework::standard::buckets::{LimitedFor, RevertBucket};
use serenity::framework::standard::macros::{check, command, group, help, hook};
use serenity::framework::standard::{
    help_commands,
    Args,
    CommandGroup,
    CommandOptions,
    CommandResult,
    DispatchError,
    HelpOptions,
    Reason,
    StandardFramework,
};
use serenity::http::Http;
use serenity::model::channel::{Channel, Message};
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::id::UserId;
use serenity::model::permissions::Permissions;
use serenity::prelude::*;
use serenity::utils::{content_safe, ContentSafeOptions};
use tokio::sync::Mutex;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler; //creates handler for responses

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected.", ready.user.name);
    }
}
//sets up main command groupage
#[group]
#[commands(about)]
struct General;
//sets up open sorcery command group
//#[group]
//#[prefix = "os"]
//#[commands(info)]
//struct OS;
//various help messages
#[help]
#[individual_command_tip = format(" Grimbot {} responding. Pass a command as an argument for more details", env::var("VERSION").expect("Expected a version in the environment."))]
#[command_not_found_text = "Could not find: '{}'."]
//when searching for commands, how deep?
#[max_levenshtein_distance(3)]
#[indentation_prefix = "+"]

//help menu filter behavior
#[lacking_permissions = "Strike"]
#[lacking_role = "Strike"]
#[wrong_channel = "Strike"]

//help commmand
async fn h(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;

    #[hook]
    async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
        if let DispatchError::Ratelimited(info) = error {
            // We notify them only once.
            if info.is_first_try {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                    .await;
            }
        }
    }
}

    #[tokio::main]
    async fn main() {
        // Configure the client with your Discord bot token in the environment.
        let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
        let http = Http::new(&token);

        // We will fetch your bot's owners and id
        let (owners, bot_id) = match http.get_current_application_info().await {
            Ok(info) => {
                let mut owners = HashSet::new();
                if let Some(team) = info.team {
                    owners.insert(team.owner_user_id);
                } else {
                    owners.insert(info.owner.id);
                }
                match http.get_current_user().await {
                    Ok(bot_id) => (owners, bot_id.id),
                    Err(why) => panic!("Could not access the bot id: {:?}", why),
                }
            },
            Err(why) => panic!("Could not access application info: {:?}", why),
        };

        let framework = StandardFramework::new()
            .configure(|c| c
                       .with_whitespace(false)
                       .on_mention(Some(bot_id))
                       .prefix(env::var("PREFIX").expect("Expected a prefix in the environment."))
                       // In this case, if "," would be first, a message would never
                       // be delimited at ", ", forcing you to trim your arguments if you
                       // want to avoid whitespaces at the start of each.
                       .delimiters(vec![", ", ","])
                       // Sets the bot's owners. These will be used for commands that
                       // are owners only.
                       .owners(owners))

        // Set a function to be called prior to each command execution. This
        // provides the context of the command, the message that was received,
        // and the full name of the command that will be called.
        //
        // Avoid using this to determine whether a specific command should be
        // executed. Instead, prefer using the `#[check]` macro which
        // gives you this functionality.
        //
        // **Note**: Async closures are unstable, you may use them in your
        // application if you are fine using nightly Rust.
        // If not, we need to provide the function identifiers to the
        // hook-functions (before, after, normal, ...).
            .before(before)
        // Similar to `before`, except will be called directly _after_
        // command execution.
            .after(after)
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
            .unrecognised_command(unknown_command)
        // Set a function that's called whenever a message is not a command.
            .normal_message(normal_message)
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
            .on_dispatch_error(dispatch_error)
        // Can't be used more than once per 5 seconds:
            .bucket("emoji", |b| b.delay(5)).await
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay applying per channel.
        // Optionally `await_ratelimits` will delay until the command can be executed instead of
        // cancelling the command invocation.
            .bucket("complicated", |b| b.limit(2).time_span(30).delay(5)
                // The target each bucket will apply to.
                .limit_for(LimitedFor::Channel)
                // The maximum amount of command invocations that can be delayed per target.
                // Setting this to 0 (default) will never await/delay commands and cancel the invocation.
                .await_ratelimits(1)
                // A function to call when a rate limit leads to a delay.
                .delay_action(delay_action)).await
        // The `#[group]` macro generates `static` instances of the options set for the group.
        // They're made in the pattern: `#name_GROUP` for the group instance and `#name_GROUP_OPTIONS`.
        // #name is turned all uppercase
            .help(&MY_HELP)
            .group(&GENERAL_GROUP)
            //.group(&OS_GROUP);

        // For this example to run properly, the "Presence Intent" and "Server Members Intent"
        // options need to be enabled.
        // These are needed so the `required_permissions` macro works on the commands that need to
        // use it.
        // You will need to enable these 2 options on the bot application, and possibly wait up to 5
        // minutes.
        let intents = GatewayIntents::all();
        let mut client = Client::builder(&token, intents)
            .event_handler(Handler)
            .framework(framework)
            .type_map_insert::<CommandCounter>(HashMap::default())
            .await
            .expect("Err creating client");

        {
            let mut data = client.data.write().await;
            data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        }

        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    }


//provides information about the running bot
#[command]
async fn about(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
        ContentSafeOptions::default().display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default().clean_channel(false).clean_role(false)
    };

    let content = format!("Responding. Grimbot version {} developed by bradley.", env::var("VERSION").expect("Expecting version in the environment."));

    msg.channel_id.say($ctx.http, &content).await?;

    Ok(())
};
