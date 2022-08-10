extern crate serde_derive;

use rand::Rng;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use std::collections::HashMap;
use serde::Serialize;
use serde::Deserialize;
use std::str;
use rustbreak::deser::Ron;
use rustbreak::FileDatabase;


#[command]
#[description = "Roll for XdX dice, or give just a number to roll in the Between Clouds system. Add 'kirin' to roll as a kirin."]
#[example = "3d6"]
#[example = "4"]
#[example = "8 kirin"]
pub async fn roll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    //cut args and split into vector
    let tmp: String = args.single::<String>()?;
    let split = tmp.split_whitespace();
    let argset = split.collect::<Vec<&str>>();
    let outcome: String;
    //determine XdX or YZE rolling
    if argset[0].contains("d") {
        let tmpsplit = argset[0].split("d");
        let dsplit = tmpsplit.collect::<Vec<&str>>();
        outcome = genroll(dsplit[0].parse::<i32>().unwrap(), dsplit[1].parse::<i32>().unwrap(), "straight");
        print!("{:?}", outcome);
    } else {
        let die_count = argset[0].parse::<i32>().unwrap();
        if argset.len() == 2 {
            outcome = genroll(die_count, 6, &argset[1].to_lowercase())
        } else {
            outcome = genroll(die_count, 6, "player");
            print!("{:?}", outcome);
        }
    }

    msg.channel_id
        .say(&ctx.http, format!("Role Result: {}", outcome))
        .await?;

    Ok(())
}

fn genroll(count: i32, sides: i32, checktype: &str) -> String { //generic roll
    let mut rng = rand::thread_rng();
    let mut rolls: Vec<i32> = [].to_vec();
    for _n in 1..=count {
        let temproll: i32 = rng.gen_range(1..=sides);
        rolls.push(temproll);
    }
    if checktype == "straight" {
        let sum: i32 = rolls.iter().sum();
        return format!("{:?} = {}", rolls, sum)
    } else {
        return succeed(rolls, checktype);
    }
    return "There was a problem completing this action.".to_string()
}

fn succeed(rolls: Vec<i32>, checktype: &str) -> String { //check for successes

    let mut numofsuc = rolls.iter().filter(|&n| *n == 6).count();

    if checktype == "kirin" {
        numofsuc += rolls.iter().filter(|&n|*n == 5).count();
    }
    if numofsuc == 1 {
        return format!("{} Success Rolled", numofsuc);
    }
    else {
        return format!("{} Successes Rolled", numofsuc);
    }
}

// CHARACTER COMMANDS (holy shit this is getting complex over here)
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Symbiote {
    user: u64,
    name: String,
    attributes: HashMap<String, u32>,
    role: String,
    mutations: HashMap<String, String>,
}

fn save(sym: Symbiote, name: String) -> Result<(), rustbreak::RustbreakError> {
    let db = FileDatabase::<HashMap<String, Symbiote>, Ron>::load_from_path_or_default("characters.ron")?;
    println!("Writing to database");
    db.write(|db| db.insert(name.into(), sym));
    println!("Saving database");
    db.save()?;

    Ok(())
}

fn load(file: String, target: String) -> CommandResult {
    Ok(())
}
