use select::document::Document;
use select::predicate::*;
fn main(){

    let htmlTXT = include_str!("Coh3exemple.html");
    
    let doc = Document::from(htmlTXT);

    for node in doc.find(Class("even:bg-[#131513]").descendant(Name("td"))) {
        for child in node.children() {
            println!("{}",child.text());
        }
    }
}


