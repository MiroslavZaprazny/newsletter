use reqwest::Client;

use crate::domain::Email;

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
    ) -> Result<(), String> {
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

        let res = self
            .client
            .post(url)
            .header("Authorization", bearer_token)
            .json(&body)
            .send()
            .await;
        println!(
            "emial response: {}",
            res.unwrap()
                .text()
                .await
                .expect("Cannot decode response body")
        );

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use wiremock::{MockServer, Mock, matchers::any, ResponseTemplate};
//     use crate::{email_client::EmailClient, domain::Email};
//
//     #[tokio::test]
//     async fn send_email_fires_a_request_to_base_url() {
//         let server = MockServer::start().await;
//         Mock::given(any()).respond_with(ResponseTemplate::new(200))
//             .expect(1)
//             .mount(&server)
//             .await;
//
//         let sender = Email::parse(String::from("test@email.com")).expect("Failed to parse email");
//         let client = EmailClient::new(server.uri(), sender);
//         let recipient = Email::parse(String::from("test12@email.com")).expect("Failed to parse email");
//
//         client.send_email(recipient, "test email", "testing").await;
//     }
// }
