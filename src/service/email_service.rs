
use actix_web::web::Data;
use chrono::{Duration, Local, NaiveDate, Timelike, TimeZone, Utc};
use lettre::{
    Message,
    SmtpTransport,
    Transport,
};
use lettre::error::Error as LettreError;
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::Error as SmtpError;
use lettre::transport::smtp::response::Response;
use log::{debug, error, info};
use mime::TEXT_HTML;
use tokio::time::sleep;

use crate::model::task_model::Task;
use crate::repository::task_repository::TaskRepository;
use crate::repository::user_repository::UserRepository;

#[derive(Debug)]
pub enum EmailError {
    Smtp(SmtpError),
    Lettre(LettreError),
}

impl From<SmtpError> for EmailError {
    fn from(error: SmtpError) -> Self {
        EmailError::Smtp(error)
    }
}

impl From<LettreError> for EmailError {
    fn from(error: LettreError) -> Self {
        EmailError::Lettre(error)
    }
}

pub async fn morning_email_scheduler(user_repo: Data<UserRepository>, task_repo: Data<TaskRepository>) {
    info!("Scheduler is active");

    loop {
        let now = chrono_tz::Europe::Bucharest.from_utc_datetime(&Utc::now().naive_utc());
        let next_morning = (now + Duration::days(1))
            .with_hour(6)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();

        let duration_until_next_morning = (next_morning - now)
            .to_std().expect("Failed to calculate duration");

        sleep(duration_until_next_morning).await;

        info!("Running scheduler for task emails");
        send_email_to_users(&user_repo, &task_repo).await;

        let seconds = duration_until_next_morning.as_secs();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        info!("Scheduler finished. Next run in {}h {}m {}s", hours, minutes, seconds);
    }
}

async fn send_email_to_users(user_repo: &Data<UserRepository>, task_repo: &Data<TaskRepository>) {
    let users = match user_repo.find_all().await {
        Ok(users) => users,
        Err(e) => {
            error!("Error in scheduler while fetching users: {}", e);
            Vec::new()
        }
    };

    let today: NaiveDate = Local::now().date_naive();
    for user in users {
        let user_tasks = match task_repo.find_by_due_date(&user.id.unwrap(), today).await {
            Ok(tasks) => tasks,
            Err(e) => {
                error!("Error in scheduler while fetching user\'s tasks: {}", e);
                Vec::new()
            }
        };

        if user_tasks.is_empty() {
            continue;
        } else {
            let subject = format!("Tasks due on {}", today);
            let body = build_html_body(&user_tasks).await;

            match send_email(&user.email, &subject, body).await {
                Ok(_) => debug!("Email sent to {} for {} tasks", user.email, user_tasks.len()),
                Err(e) => error!("Error sending email to {}: {:?}", user.email, e)
            };
        }
    }
}

async fn send_email(to: &String, subject: &String, body: String) -> Result<Response, EmailError> {
    let smtp_username = std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME not provided");
    let smtp_password = std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD not provided");

    let from_mailbox: Mailbox = smtp_username.parse().unwrap();
    let to_mailbox: Mailbox = to.parse().unwrap();
    let message_result = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .header(ContentType::parse(TEXT_HTML.as_ref()).unwrap())
        .subject(subject)
        .body(body);

    let message = match message_result {
        Ok(msg) => msg,
        Err(e) => return Err(EmailError::Lettre(e)),
    };

    let credentials = Credentials::new(smtp_username, smtp_password);
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(credentials)
        .build();

    mailer.send(&message).map_err(EmailError::from)
}

async fn build_html_body(tasks: &Vec<Task>) -> String {
    let mut body = String::from("<html><body>");
    body.push_str("<h1>Tasks Due Today</h1>");
    body.push_str("<ul>");

    for task in tasks {
        body.push_str(&format!("<li><b>{}</b> -> {} -> [In: {}]</li>", task.title, task.description, task.status));
    }

    body.push_str("</ul>");
    body.push_str("</body></html>");

    body
}