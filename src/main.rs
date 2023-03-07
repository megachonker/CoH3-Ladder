use select::document::Document;
use select::predicate::*;

extern crate reqwest;

#[tokio::main]
async fn main(){

    let url = "https://leaderboards.companyofheroes.com/";

    let resp = reqwest::get(url).await.unwrap();
    assert!(resp.status().is_success());

    let cat  = resp.text().await.unwrap();

    let ccat = cat.as_str();

    // println!("{}",ccat);

    // #[tokio::main]
    // async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //     let resp = reqwest::get("https://httpbin.org/ip")
    //         .await?
    //         .json::<HashMap<String, String>>()
    //         .await?;
    //     println!("{:#?}", resp);
    //     Ok(())
    // }

    // let mut file = File::open("Coh3exemple.html")?;
    // let mut html = String::new();
    // file.read_to_string(&mut html)?;
    
    // let htmlTXT = resp.;//include_str!("Coh3exemple.html");
    
    let doc = Document::from(ccat);

    for node in doc.find(Attr("class", "even:bg-[#131513]")) {
        println!("{}",node.text());
        
        for nono in node.children(){
            print!("{},", nono.text());
        }
    }
    
}


