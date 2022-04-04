use crate::domain::new_user::ForgotPassword;

pub struct EmailClient {
    sender: ForgotPassword,
}
impl EmailClient {
    pub async fn send_email(
        &self,
        _recipient: ForgotPassword,
        _subject: &str,
        _html_content: &str,
        _text_content: &str,
    ) -> Result<(), String> {
        println!("{}", self.sender.email_address);
        Ok(())
    }
}
