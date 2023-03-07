// use std::fs::File;
// use std::io::prelude::*;

use select::document::Document;
use select::predicate::*;
fn main(){

    // let mut file = File::open("Coh3exemple.html")?;
    // let mut html = String::new();
    // file.read_to_string(&mut html)?;
    
    let htmlTXT = include_str!("Coh3exemple.html");
    
    let doc = Document::from(htmlTXT);

    for node in doc.find(Attr("class", "even:bg-[#131513]").descendant(Name("td"))) {
        println!("{} ({:?})\n", node.text(), node.attr("class").unwrap());
    }

}


