#[derive(Debug, Serialize, Deserialize, Clone)]
struct Group {
    users: Vec<u64>,
    name: String,
    answers: HashMap<u64, bool>,
    set: bool,
    date: String,
}

pub async fn group(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let tmp: String = args.single::<String>()?;
    print!("{}", tmp);
    let split = tmp.split_whitespace();
    let argset = split.collect::<Vec<&str>>();
    if argset[0] == "new" && argset.len() > 1 {
        msg.channel_id
            .say(&ctx.http, format!("Creating new group {:?}", argset[1]))
            .await?;

        let new_grp = Group {
            users: vec![msg.author.id.0],
            name: argset[1].to_string(),
            answers: HashMap::from_iter(vec![(msg.author.id.0, false)]),
            set: false,
            date: String::new(),
        };

        msg.channel_id
            .say(&ctx.http, format!("Group created: {:?}", new_grp))
            .await?;

        let recon_user = serenity::model::id::UserId(msg.author.id.0);
        let tar = recon_user.create_dm_channel(&ctx.http).await?;

        tar.say(&ctx.http, "I've found you").await?;
        save_group(new_grp, argset[1].to_string());
    } else {
        msg.channel_id.say(&ctx.http, "Else").await?;
    }

    Ok(())
}

use rustbreak::deser::Ron;
use rustbreak::FileDatabase;

fn save_group(grp: Group, name: String) -> Result<(), rustbreak::RustbreakError> {
    let db = FileDatabase::<HashMap<String, Group>, Ron>::load_from_path_or_default("groups.ron")?;
    println!("Writing to database");
    db.write(|db| db.insert(name.into(), grp));
    println!("Saving database");
    db.save()?;

    Ok(())
}

fn load(file: String, target: String) -> CommandResult {
    Ok(())
}
