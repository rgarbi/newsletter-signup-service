use crate::domain::new_user::ForgotPassword;

pub struct EmailClient {
    sender: ForgotPassword,
}
impl EmailClient {
    pub async fn send_email(
        &self,
        recipient: ForgotPassword,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
