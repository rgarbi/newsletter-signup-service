use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::domain::valid_email::ValidEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: ValidEmail,
    api_key: Secret<String>,
}
impl EmailClient {
    pub fn new(base_url: String, sender: ValidEmail, api_key: Secret<String>) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            api_key,
        }
    }

    pub async fn send_email(
        &self,
        recipient: ValidEmail,
        subject: &str,
        _html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let auth_header = format!("Bearer {}", self.api_key.expose_secret());

        let contents: [Personalization; 1] = [Personalization {
            to: [SendTo {
                email: recipient.to_string(),
            }; 1],
            from: SendFrom {
                email: self.sender.to_string(),
            },
            subject: String::from(subject),
            content: [EmailContent {
                content_type: "text/plain".to_string(),
                value: text_content.to_string(),
            }; 1],
        }];

        let email_content = SendEmailRequest {
            personalizations: contents,
        };

        let address = format!("{}/v3/mail/send", &self.base_url);

        let result = self
            .http_client
            .post(address)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .body(email_content.to_json())
            .send()
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct SendEmailRequest {
    pub personalizations: [Personalization; 1],
}

#[derive(Deserialize, Serialize)]
struct Personalization {
    pub to: [SendTo; 1],
    pub from: SendFrom,
    pub subject: String,
    pub content: [EmailContent; 1],
}

#[derive(Deserialize, Serialize)]
struct SendTo {
    pub email: String,
}

#[derive(Deserialize, Serialize)]
struct SendFrom {
    pub email: String,
}

#[derive(Deserialize, Serialize)]
struct EmailContent {
    #[serde(rename(serialize = "type", deserialize = "content_type"))]
    #[serde(alias = "content_type", alias = "type")]
    pub content_type: String,
    pub value: String,
}

impl SendEmailRequest {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Was not able to serialize.")
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::Fake;
    use secrecy::Secret;
    use wiremock::matchers::{header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use crate::domain::valid_email::ValidEmail;
    use crate::email_client::{EmailClient, SendEmailRequest};

    struct SendEmailBodyMatcher;
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let body = request.body.clone();
            let email_request: SendEmailRequest =
                serde_json::from_str(String::from_utf8(body).unwrap().as_str()).unwrap();

            let size_is_one: bool = email_request.personalizations.len() == 1;
            let has_subject: bool = !email_request.personalizations[0].subject.is_empty();
            let has_content: bool = email_request.personalizations[0].content.len() == 1;
            size_is_one && has_subject && has_content
        }
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = ValidEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(String::new()));
        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("v3/mail/send"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        let subscriber_email = ValidEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();
        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
        // Assert
    }
}
