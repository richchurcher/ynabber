use std::error::Error;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Todo {
    user_id: i32,
    id: i32,
    title: String,
    completed: bool,
}

pub struct BudgetAPI {
    access_token: String,
    client: reqwest::Client,
}

impl BudgetAPI {
    pub fn new(access_token: &str) -> Result<BudgetAPI, Box<dyn Error>> {
        Ok(BudgetAPI {
            access_token: access_token.to_owned(),
            client: reqwest::Client::new(),
        })
    }

    pub async fn request(&mut self, url: &str) -> Result<(), Box<dyn Error>> {
        let todo = self.client.get(url).send().await?.json::<Todo>().await?;
        println!("{:?}", todo);
        Ok(())
    }
}
