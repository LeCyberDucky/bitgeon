// use fancy_regex::Regex;
// use lazy_static::lazy_static;
// use std::io;
// use std::thread;
// use std::time;
// use unicode_segmentation::Graphemes;
// use unicode_segmentation::UnicodeSegmentation;

fn main() {
    let trim_characters = [';', '\"'];
    let path = String::from("Howdy!\"");

    let path = path.trim_matches(|c: char| c.is_whitespace() || trim_characters.contains(&c));

    // let mut A = vec![1, 2, 3, 4, 5];
    // let B = vec![10, 11, 4, 5, 6, 6,5, 4, 77, 6,5 ];
    // A.splice(4..5, B);

    // println!("{:?}", A);
}

// fn main() {
//     let dings =
//         r"C:\Users\Andy\Desktop\Test.txt C:\Users\Andy\Desktop\Test C:\Users\Andy\Desktop\Test 1\";

//     lazy_static! {
//         static ref re: Regex =
//             Regex::new(r"((?:[A-z]:.+?(?=[A-z]:|$|\n|;))|(?:.+?(?=;|$|\n|[A-z]:)))").unwrap();
//     }

//     // let result = re.captures(dings);

//     // let captures = result.expect("Error running regex").expect("No match found");
//     // let group = captures.get(0).expect("No group");

//     // let match_1 = re.captures_from_pos(&dings, 0);
//     // let match_1 = match_1.expect("Error running regex").expect("No match found");
//     // let match_1 = match_1.get(0).expect("No group");
//     // println!("{}", match_1.as_str());

//     // let match_2 = re.captures_from_pos(&dings, match_1.end());
//     // let match_2 = match_2.expect("Error running regex").expect("No match found");
//     // let match_2 = match_2.get(0).expect("No group");
//     // println!("{}", match_2.as_str());

//     // let match_3 = re.captures_from_pos(&dings, match_2.end());
//     // let match_3 = match_3.expect("Error running regex").expect("No match found");
//     // let match_3 = match_3.get(0).expect("No group");
//     // println!("{}", match_3.as_str());

//     // println!("{}", r"C:\Users\Andy\Desktop\Test.txt ".len());

//     let mut capture_pos = 0;
//     while capture_pos < dings.len() {
//         let result = re.captures_from_pos(&dings, capture_pos);
//         let captures = result
//             .expect("Error running regex")
//             .expect("No match found");
//         let group = captures.get(0).expect("No gorup");
//         capture_pos = group.end();
//         println!("{}", group.as_str());
//     }

//     // for i in 0..captures.len() {
//     //     if let Some(daddel) = captures.get(i) {
//     //         println!("[{}..{}] \"{}\"", daddel.start(), daddel.end(), daddel.as_str());
//     //     }
//     //     // println!("{}", captures.get(i).as_str());
//     // }
// }
