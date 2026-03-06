use mailtrap_rs::{
    client::MailtrapClient,
    types::email::{Body, EmailAddress},
};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

use crate::configuration::EmailClientSettings;
use crate::domain::valid_email::ValidEmail;

#[derive(Clone)]
pub struct EmailClient {
    mailtrap_client: MailtrapClient,
    from_email: EmailAddress,
    reply_to_email: EmailAddress,
}

impl EmailClient {
    pub fn new(email_settings: EmailClientSettings) -> Self {
        Self {
            mailtrap_client: MailtrapClient::new(
                &email_settings.base_url,
                email_settings.api_key.expose_secret().to_string(),
                std::time::Duration::from_millis(email_settings.timeout_milliseconds),
            )
            .expect("Invalid Mailtrap client configuration"),
            from_email: EmailAddress::new(
                email_settings.sender_email,
                Some(email_settings.sender_name),
            )
            .expect("Invalid from email address"),
            reply_to_email: EmailAddress::new(
                email_settings.reply_to_email,
                Some(email_settings.reply_to_name),
            )
            .expect("Invalid reply to email address"),
        }
    }

    #[tracing::instrument(
        name = "Sending an email",
        skip(self, recipient, subject, html_content, text_content)
    )]
    pub async fn send_email(
        &self,
        recipient: Vec<ValidEmail>,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), anyhow::Error> {
        let to_addresses: Vec<EmailAddress> = recipient
            .iter()
            .map(|r| {
                EmailAddress::new(r.to_string(), None)
                    .expect("ValidEmail should produce valid email")
            })
            .collect();

        let message = mailtrap_rs::types::email::Message::new(
            self.from_email.clone(),
            subject.to_string(),
            Body::TextAndHtml {
                text: text_content.to_string(),
                html: html_content.to_string(),
            },
        )
        .reply_to(self.reply_to_email.clone());

        let message = to_addresses
            .into_iter()
            .fold(message, |msg, addr| msg.to(addr));

        self.mailtrap_client
            .send_email(message)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct SendEmailRequest {
    pub personalizations: Vec<Personalization>,
    pub from: SendFrom,
    pub subject: String,
    pub content: [EmailContent; 2],
}

#[derive(Deserialize, Serialize)]
pub struct Personalization {
    pub to: Vec<SendTo>,
}

#[derive(Deserialize, Serialize)]
pub struct SendTo {
    pub email: String,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct SendFrom {
    pub email: String,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct EmailContent {
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
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::SecretString;
    use wiremock::matchers::{any, header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::configuration::EmailClientSettings;
    use crate::domain::valid_email::ValidEmail;
    use crate::email_client::{from_recipient_to_personalizations, EmailClient};

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> ValidEmail {
        ValidEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        let base_url_with_slash = if base_url.ends_with('/') {
            base_url
        } else {
            format!("{}/", base_url)
        };
        EmailClient::new(EmailClientSettings {
            base_url: base_url_with_slash,
            sender_email: email().to_string(),
            sender_name: "Test".to_string(),
            reply_to_email: email().to_string(),
            reply_to_name: "Test".to_string(),
            api_key: SecretString::new(Faker.fake::<String>().into_boxed_str()),
            timeout_milliseconds: 200,
        })
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        Mock::given(header_exists("Api-Token"))
            .and(path("api/send"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{"success":true,"message_ids":["test-id"],"errors":[]}"#,
                "application/json",
            ))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(Vec::from([email()]), &subject(), &content(), &content())
            .await;
        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(Vec::from([email()]), &subject(), &content(), &content())
            .await;
        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_errors_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500).set_delay(std::time::Duration::from_secs(180)))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(Vec::from([email()]), &subject(), &content(), &content())
            .await;
        // Assert
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn from_recipient_to_personalizations_works() {
        let expected = 100;
        let mut valid_emails: Vec<ValidEmail> = Vec::new();
        for _i in 0..expected {
            valid_emails.push(ValidEmail::parse(SafeEmail().fake()).unwrap())
        }

        let transformed = from_recipient_to_personalizations(valid_emails);

        assert_eq!(expected, transformed.len());
    }
}
