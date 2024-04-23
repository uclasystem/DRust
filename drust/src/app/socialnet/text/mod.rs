use crate::drust_std::collections::{dstring::{DString, DStringMut}, dvec::DVec};

pub async fn text_service(mut text: DStringMut<'_>) -> DVec<DString> {
    // println!("Received text: {}", text);
    let mut mentions = DVec::new();
    // let mut urls = DVec::new();
    let length = text.len();
    let text_mut = text.as_mut();
    let mut i = 0;
    while i < length {
        // if i < length - 8 &&
        //     (text_mut[i..i + 7] == "http://".as_bytes()[..] ||
        //         text_mut[i..i + 8] == "https://".as_bytes()[..])
        // {
        //     let mut j = i + 8;
        //     while j < length && text_mut[j] != b' ' {
        //         j += 1;
        //     }
        //     let mut url = DString::with_capacity(j - i);
        //     for k in i..j {
        //         url.push(text_mut[k]);
        //     }
        //     urls.push(url);
        //     i = j;
        //     continue;
        // }
        if text_mut[i] == b'@' {
            let mut j = i;
            while j < length && text_mut[j] != b' ' {
                j += 1;
            }
            let mut mention = DString::with_capacity(j - i);
            for k in i..j {
                mention.push(text_mut[k]);
            }
            mentions.push(mention);
            i = j;
        }
        i += 1;
    }
    mentions
}
