use reqwest::Client;

use crate::domain::Email;

#[derive(Debug)]
pub struct EmailClient {
    client: Client,
    url: String,
    sender: Email,
    auth_code: String,
}

#[derive(serde::Serialize)]
struct Personalizations {
    to: [To; 1],
}

#[derive(serde::Serialize)]
struct To {
    email: String,
}

#[derive(serde::Serialize)]
struct From {
    email: String,
}

#[derive(serde::Serialize)]
struct Content {
    value: String,
    r#type: String,
}

#[derive(serde::Serialize)]
struct SendEmailPayload {
    personalizations: [Personalizations; 1],
    from: From,
    subject: String,
    content: [Content; 1],
}

impl EmailClient {
    pub fn new(url: String, sender: Email, auth_code: String) -> Self {
        Self {
            client: Client::new(),
            url,
            sender,
            auth_code,
        }
    }

    pub async fn send_email(
        &self,
        recipient: Email,
        subject: &str,
        body: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/mail/send", self.url);
        let body = SendEmailPayload {
            personalizations: [Personalizations {
                to: [To {
                    email: recipient.as_ref().to_owned(),
                }],
            }],
            from: From {
                email: self.sender.as_ref().to_owned(),
            },
            subject: subject.to_owned(),
            content: [Content {
                value: body.to_owned(),
                r#type: String::from("text/html"),
            }],
        };
        let bearer_token = format!("Bearer {}", self.auth_code);

        self.client
            .post(url)
            .header("Authorization", bearer_token)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{domain::Email, email_client::EmailClient};
    use wiremock::{
        matchers::{header, header_exists, method, path},
        Match, Mock, MockServer, ResponseTemplate,
    };

    struct SendEmailMatcher;

    impl Match for SendEmailMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, serde_json::Error> = request.body_json();
            println!("body json: {:?}", result);
            if let Ok(body) = result {
                body.get("content").is_some()
                    && body.get("from").is_some()
                    && body.get("subject").is_some()
                    && body.get("personalizations").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let server = MockServer::start().await;
        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("/mail/send"))
            .and(method("POST"))
            .and(SendEmailMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let sender = Email::parse(String::from("test@email.com")).expect("Failed to parse email");
        let auth_code = String::from("123authcode");
        let client = EmailClient::new(server.uri(), sender, auth_code);
        let recipient =
            Email::parse(String::from("test12@email.com")).expect("Failed to parse email");

        let res = client.send_email(recipient, "test email", "testing").await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        let server = MockServer::start().await;
        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("/mail/send"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;

        let sender = Email::parse(String::from("test@email.com")).expect("Failed to parse email");
        let auth_code = String::from("123authcode");
        let client = EmailClient::new(server.uri(), sender, auth_code);
        let recipient =
            Email::parse(String::from("test12@email.com")).expect("Failed to parse email");

        let res = client.send_email(recipient, "test email", "testing").await;

        assert!(res.is_err());
    }
}
