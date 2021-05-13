pub struct WeIdRestService{
    endpoint_url: String,
}

impl WeIdRestService{
    pub fn new(endpoint_url: String) -> WeIdRestService {
        WeIdRestService { endpoint_url }
    }
    pub fn get_endpoint_url(&self) -> String {
        self.endpoint_url.to_string()
    }

    // pub fn get_block_number(&self) -> Result<String, reqwest::Error>{
    //     let mut url =self.ip.to_string();
    //     url += &"WeBASE-Front/1/web3/blockNumber/".to_string();
    //     let resp = 
    //         reqwest::blocking::get(&url)?
    //         // .await?
    //         .text();
    //         // .await?;

    //     resp
    // }
}