use serde_json::{Value};
use thiserror::Error;

/// Provide an implementation for the default() method:
/// https://doc.rust-lang.org/stable/core/default/trait.Default.html
#[derive(Default)]
pub struct WeIdGenerator{
    endpoint_url: String,
    weid: String, 
}

impl WeIdGenerator{
    pub fn new(endpoint_url: String) -> WeIdGenerator {
        WeIdGenerator {endpoint_url, ..Default::default()}
    }
    /// String or &str?
    /// Ref: https://zhuanlan.zhihu.com/p/123278299
    /// 显然，这取决于很多因素，但是一般地，保守来讲，如果我们正在构建的API不需要拥有或者修改使用的文本，
    /// 那么应该使用&str而不是String。
    /// 等一下，但是如果这个API的调用者真的有一个String并且出于某些未知原因无法将其转换成&str呢？完全没有问题。
    /// Rust有一个超级强大的特性叫做deref coercing，这个特性能够允许把传进来的带有借用操作符的String引用，
    /// 也就是&String，在API执行之前转成&str。我们会在另一篇文章里介绍更多地相关细节。
    pub fn generate_local(&mut self, chain_id: i32, addr: &str) -> String {
        self.weid = "did:weid:".to_string() + &chain_id.to_string() + ":" + addr;
        // Ref: https://stackoverflow.com/questions/38304666/how-to-define-a-copyable-struct-containing-a-string
        // String is copyable, use .clone()
        // String is not implicitly copyable, because that would cause non-obvious memory allocations to occur
        self.weid.clone()
    }

    /// create weid online.
    pub fn create_weid_online(&self) -> Result<Value, GenerateWeIdError>{
        let response = self.call_create_weid()?;
        let resp = self.str_to_json(&response)?;
        Ok(resp)
    }

    fn str_to_json(&self, payload: &str) -> Result<Value, serde_json::Error> {
        serde_json::from_str(payload)
    }
    pub fn call_create_weid(&self) -> Result<String, reqwest::Error> {
        let mut url =self.endpoint_url.to_string();
        url += &"/weid/api/invoke".to_string();
        // ::blocking:: to block
        let response = reqwest::blocking::Client::new()
        .post(&url)
        .json(&serde_json::json!({
            "functionArg": {},
            "transactionArg": {},
            "v": "1.0.0",
            "functionName": "createWeId"
        }))
        .send()?
        .text();
        
        response
    }
}
/// multi error handle:
/// https://my.oschina.net/jmjoy/blog/3190024
#[derive(Error, Debug)]
pub enum GenerateWeIdError {
    #[error("req error")]
    RequestError(#[from] reqwest::Error),
    #[error("parse error")]
    ParseError(#[from] serde_json::Error),
}

