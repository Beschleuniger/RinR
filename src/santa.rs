use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};

use serenity::all::{Context, Message, UserId};


use crate::helper::Santa;





pub async fn santaHandler(msg: &Message, ctx: &Context) {
    
    // Aquire lock for global data
    let mut u_data = ctx.data.write().await;
    let santa: &mut Santa = u_data.get_mut::<Santa>().expect("No Santa Vector Available");

    let m: String = msg.content.clone();
    
    if m.contains("register") {
        santa.members.insert(msg.author.id.clone());
    } else if m.contains("remove") {
        santa.members.remove(&msg.author.id);
    } else if m.contains("start") {

        if santa.members.len() < 2 {
            return;
        }

        println!("Current Members: {:#?}", santa);


        let mut gifter: Vec<UserId> = Vec::with_capacity(8);
        let mut giftee: Vec<UserId> = Vec::with_capacity(8);

        gifter.extend(santa.members.clone().into_iter());
        giftee.extend(santa.members.clone().into_iter());

        let mut rng = StdRng::from_entropy();

        loop {
            gifter.shuffle(&mut rng);
            giftee.shuffle(&mut rng);
        
            if !gifter.iter().zip(&giftee).any(|(l, r)| l == r) {
                break;
            }
        }

        for (gift, rec) in gifter.iter().zip(giftee.iter()) {
            dmUser(gift, rec, ctx).await;
        }


    } else {
        println!("Current Members: {:#?}", santa);
    }

}

async fn dmUser(gift: &UserId, rec: &UserId, ctx: &Context) {

    let msg = format!("Your Secret Santa is <@{}>", rec);

    if let Ok(user) =  gift.to_user(&ctx.http).await {
        if let Ok(channel) = user.create_dm_channel(&ctx.http).await {
            match channel.say(&ctx.http, msg).await {
                Ok(_) =>println!("Successfully sent message to user {}:{} -> {}", user.name, user.id, rec.get()),
                Err(_) => println!("Failed to send message to {}", user.name),
            }
        }        
    }
}