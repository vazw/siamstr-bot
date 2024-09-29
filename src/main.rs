use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use nostr_sdk::prelude::*;

pub async fn get_bitcoin_block() -> u32 {
    let mut block: Option<u32> = None;
    while block.is_none() {
        let req = reqwest::get("https://blockchain.info/q/getblockcount").await;
        if let Ok(res) = req {
            block = Some(res.json::<u32>().await.expect("block height"));
        }
    }
    block.unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    let sk = "nsec here";
    // u32 represent count for manaully get reaction from unsupported COUNT relay
    let mut notes: HashMap<EventId, (Event, u32)> = HashMap::new();
    let client = Client::default();
    // TODO ADD MORE RELAY
    _ = client.add_relay("wss://relay.siamstr.com").await;
    if client.add_relay("ws://locahost:4869").await.is_ok() {
        println!("Connected to local relay");
    }
    client.connect().await;

    // โน๊ตจาก 24 ชั่วโมงล่าสุด
    let from_time =
        Timestamp::from_secs(Timestamp::now().as_u64() - (3600 * 24));
    // filter โน๊ต 24 ชม ล่าสุดที่มี #siamstr
    let filter = Filter::new()
        .kind(Kind::TextNote)
        .hashtag("siamstr")
        .since(from_time)
        .remove_limit();

    // let sub_id = client.subscribe(vec![filter], None).await?;
    let mut rx = client
        .stream_events_of(vec![filter], Some(Duration::from_secs(15)))
        .await?;

    while let Some(event) = rx.next().await {
        notes.insert(event.id, (event, 0));
    }

    // สร้าง filter เอาไว้นับจำนวน reaction ใน note -> nip-25, nip-45
    // let filter_count = Filter::new().ids(notes.keys().map(|x| x.to_owned()));
    let filter_count = Filter::new()
        .kinds(vec![Kind::Reaction, Kind::Repost, Kind::TextNote])
        .custom_tag(
            SingleLetterTag {
                character: Alphabet::E,
                uppercase: false,
            },
            notes.keys().map(|x| x.to_owned()),
        );
    let mut counter = client
        .stream_events_of(vec![filter_count], Some(Duration::from_secs(10)))
        .await?;

    // filter kind reaction from note_id and add count to our map
    println!("Start counting");
    while let Some(event) = counter.next().await {
        // มันจะไปมีได้ไง 5555555555555555555555555
        let e_tags = event.tags;
        for tag in e_tags {
            if tag.kind().eq(&TagKind::SingleLetter(
                SingleLetterTag::from_char('e').expect("event tag"),
            )) {
                if let Some(fid) = tag.content() {
                    if let Some((_, count_n)) = notes.get_mut(
                        &EventId::from_str(fid).expect("is a valid event id"),
                    ) {
                        *count_n += 1;
                    }
                }
            }
        }
    }
    // sort by count ascending
    let mut sort_note = notes.iter().collect::<Vec<_>>();
    sort_note.sort_by(|x, y| y.1 .1.cmp(&x.1 .1));
    println!("sorted");

    // เอามาแค่ 10 อันดับแรก
    let trending = sort_note
        .iter()
        .map(|x| x.0.to_bech32().unwrap())
        .take(10)
        .collect::<Vec<_>>();
    let trending_text = trending.join("\n\n");

    let block = get_bitcoin_block().await;
    // let content = format!(
    //     "[BOT] {block}
    // สวัสดีตอนเที่ยง น้องวัวได้รวบรวมโน๊ตที่ท่านอาจจะพลาดไป ลองไปชมกันเลย!
    // {trending_text}
    // #siamstr"
    // );
    let content = format!(
        "[BOT] {block}
สวัสดีตอนเที่ยง น้องวัวได้รวบรวมโน๊ตที่ท่านอาจจะพลาดไป ลองไปชมกันเลย!

{trending_text}
#siamstr"
    );
    println!("{content}");

    // # events = EventBuilder.text_note(
    // #     f"""[BOT] {block}
    // # สวัสดีตอนเที่ยง น้องวัวได้รวบรวมโน๊ตที่ท่านอาจจะพลาดไป ลองไปชมกันเลย!
    // # {pub_event}
    // # #siamstr""",
    // #     [Tag.hashtag("siamstr")],
    // # )
    // # print(events.to_event(app_key).as_json())

    let sk = SecretKey::from_bech32(sk).expect("NSEC is valid");
    let keys = Keys::new(sk);
    let opt = Options::default();
    // น้องวัว client
    let client = Client::with_opts(keys.clone(), opt);
    _ = client.add_relay("wss://relay.siamstr.com/").await;
    _ = client.add_relay("wss://relay.notoshi.win/").await;
    _ = client.add_relay("wss://relay.damus.io/").await;
    _ = client.add_relay("wss://nos.lol/").await;
    client.connect().await;
    let e_id = client
        .publish_text_note(content, [Tag::hashtag("siamstr")])
        .await?;
    println!("Sent Event! {e_id:?}");

    Ok(())
}
