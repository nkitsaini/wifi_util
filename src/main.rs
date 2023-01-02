use anyhow::Context;
use clap::Parser;
use soup::prelude::*;
use reqwest::Url;
use std::{env, collections::HashMap};

#[derive(Debug, Clone, Copy, clap::Subcommand)]
enum Action {
    Login,
    Restart
}

/// Simple program to deal with syrotech routers
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Origin for syrotech router. Request will made using `http` only
    #[arg(default_value_t={"192.168.1.1".to_string()})]
    syrotech_origin: String,

    /// username to login with
    #[arg(short, long, default_value_t={"admin".to_string()})]
    user: String,

    /// password to login with
    #[arg(short, long, default_value_t={"admin@123".to_string()})]
    pass: String,

    /// Should the router be restarted.
    /// By default it only logs in
    #[arg(short, long, default_value_t=false)]
    restart: bool
}

struct Creds {
    user: String,
    pass: String
}

impl Creds {
    pub fn new(user: String, pass: String) -> Self {
        Self { user, pass }
    }
}

struct SyrotechAdapter {
    origin: url::Url,
    creds: Creds,
    client: reqwest::Client
}

impl SyrotechAdapter {
    pub fn new(origin: url::Url, creds: Creds) -> Self {
        Self {
            origin,
            creds,
            client: reqwest::Client::new()
        }
    }

    fn url(&self, path: &str) -> anyhow::Result<url::Url> {
        Ok(self.origin.clone().join(path)?)
    }

    /// returns false if already logged in, otherwise logs in and returns true
    async fn log_in(&self) -> anyhow::Result<bool> {
        let r = self.client.get(self.url("/")?).send().await?;
        if r.url().path() != "/admin/login_en.asp" {
            return Ok(false);
        }
        let r = r.text().await?;

        let soup = Soup::new(&r);

        // Captch Code
        let r = soup.attr("id", "check_code").find().context("#check_code missing in html")?;
        let attrs = r.attrs();
        let captcha_value = attrs.get("value").context("#check_code does not contain value")?;

        // Csrf token
        let r = soup.attr("name", "csrftoken").find().context("csrf_token missing in html")?;
        let attrs = r.attrs();
        let csrf_token = attrs.get("value").context("csrftoken does not contain value")?;


        let mut form_params = HashMap::new();
        form_params.insert("username", self.creds.user.clone());
        form_params.insert("psd", self.creds.pass.clone());
        form_params.insert("verification_code", captcha_value.to_string());
        form_params.insert("csrftoken", csrf_token.to_string());

        let r = self.client.post(self.url("/boaform/admin/formLogin_en")?).form(&form_params).send().await?;

        // Redirects to home page on successfull login
        assert!(r.url().path() == "/", "Login failed");

        Ok(true)
    }

    async fn restart(&self) -> anyhow::Result<()> {
        // No worry about network resource in case of router. Can make many requests
        self.log_in().await?;
        let r = self.client.get(self.url("/mgm_dev_reboot_en.asp")?).send().await?.text().await?;

        let soup = Soup::new(&r);

        let r = soup.attr("name", "csrftoken").find().context("csrf_token missing in html")?;
        let attrs = r.attrs();
        let csrf_token = attrs.get("value").context("csrftoken does not contain value")?;

        let mut form_params = HashMap::new();
        form_params.insert("submit-url", "/mgm_dev_reboot_en.asp".to_string());
        form_params.insert("csrftoken", csrf_token.to_string());

        self.client.post(self.url("/boaform/admin/formReboot")?).form(&form_params).send().await?;

        // TODO: add ability to wait for restart
        println!("Triggered restart. It'll probably finish within next 60 seconds");

        Ok(())
    }


}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args { syrotech_origin, restart: should_restart, user, pass } = Args::parse();


    let url: Url = format!("http://{syrotech_origin}").parse().expect("Not a valid origin");
    let syro = SyrotechAdapter::new(url, Creds {user, pass});
    if should_restart {
        syro.restart().await?;
    } else {
        if !syro.log_in().await? {
            println!("Already logged in.")
        } else {
            println!("Successfully logged in")
        };
    }
    Ok(())
}
