// src/service/auth.rs

use chrono::Utc;
use crate::Config;
use crate::database::postgres_repository::PostgresRepository;
use crate::error::app_error::AppError;
use crate::models::audit::audit_events;
use crate::models::rate_limit::RateLimitStatus;
use uuid::Uuid;

/// What happened during a login attempt.
#[allow(dead_code)]
pub enum LoginOutcome {
    /// Credentials valid and session created successfully.
    Success { session_id: Uuid, user_id: Uuid },
    /// Credentials valid but 2FA code is required before a session is issued.
    TwoFactorRequired,
}

#[allow(dead_code)]
pub struct AuthService<'a> {
    pub repo: &'a PostgresRepository,
    pub config: &'a Config,
}

impl<'a> AuthService<'a> {
    /// Checks the pre-login rate limit for a user (by id) and IP.
    /// Returns `Ok(())` if the request is allowed, or an `Err(AppError)` if
    /// it should be rejected (Delayed or Locked).
    pub async fn check_login_rate_limit(
        &self,
        user_id: Option<&Uuid>,
        ip: &str,
        user_email: Option<&str>,
        user_name: Option<&str>,
    ) -> Result<(), AppError> {
        let status = self.repo.check_login_rate_limit(user_id, ip).await?;
        match status {
            RateLimitStatus::Delayed { until } => {
                let seconds_remaining = (until - Utc::now()).num_seconds().max(0);
                Err(AppError::TooManyAttempts {
                    retry_after_seconds: seconds_remaining,
                    message: "Too many failed attempts. Please wait before trying again."
                        .to_string(),
                })
            }
            RateLimitStatus::Locked { until, can_unlock } => {
                if can_unlock
                    && self.config.login_rate_limit.enable_email_unlock
                    && let Some(uid) = user_id
                    && let Some(email) = user_email
                    && let Some(name) = user_name
                    && let Ok(token) = self.repo.create_unlock_token(uid).await
                {
                    let email_service =
                        crate::service::email::EmailService::new(self.config.email.clone());
                    let _ = email_service
                        .send_account_locked_email(
                            email,
                            name,
                            &uid.to_string(),
                            &token,
                            &self.config.login_rate_limit.frontend_unlock_url,
                        )
                        .await;
                }
                Err(AppError::AccountLocked {
                    locked_until: until,
                    message: "Account temporarily locked due to too many failed attempts. Check your email for unlock instructions.".to_string(),
                })
            }
            RateLimitStatus::Allowed => Ok(()),
        }
    }

    /// Records a failed password attempt, writes an audit log, and returns the
    /// appropriate error (rate-limited variants take priority over plain 401).
    pub async fn handle_failed_password(
        &self,
        user_id: &Uuid,
        user_email: &str,
        user_name: &str,
        ip: &str,
        client_ip: Option<String>,
        user_agent: Option<String>,
    ) -> AppError {
        let new_status = self
            .repo
            .record_failed_login_attempt(Some(user_id), ip, &self.config.login_rate_limit)
            .await;

        let _ = self
            .repo
            .create_security_audit_log(
                Some(user_id),
                audit_events::LOGIN_FAILED,
                false,
                client_ip.clone(),
                user_agent.clone(),
                Some(serde_json::json!({"reason": "invalid_password"})),
            )
            .await;

        match new_status {
            Ok(RateLimitStatus::Delayed { until }) => {
                let seconds_remaining = (until - Utc::now()).num_seconds().max(0);
                AppError::TooManyAttempts {
                    retry_after_seconds: seconds_remaining,
                    message: "Too many failed attempts. Please wait before trying again.".to_string(),
                }
            }
            Ok(RateLimitStatus::Locked { until, can_unlock }) => {
                if can_unlock && self.config.login_rate_limit.enable_email_unlock {
                    if let Ok(token) = self.repo.create_unlock_token(user_id).await {
                        let email_service =
                            crate::service::email::EmailService::new(self.config.email.clone());
                        let _ = email_service
                            .send_account_locked_email(
                                user_email,
                                user_name,
                                &user_id.to_string(),
                                &token,
                                &self.config.login_rate_limit.frontend_unlock_url,
                            )
                            .await;
                    } else {
                        tracing::warn!("Failed to create unlock token for user {}", user_id);
                    }
                }
                AppError::AccountLocked {
                    locked_until: until,
                    message: "Account temporarily locked due to too many failed attempts. Check your email for unlock instructions.".to_string(),
                }
            }
            _ => AppError::InvalidCredentials,
        }
    }

    /// Verifies a 2FA code (TOTP or backup) for the given user.
    /// Returns `Ok(backup_used)` on success, or an `Err(AppError)` on failure.
    pub async fn verify_two_factor(
        &self,
        user_id: &Uuid,
        two_factor_data: crate::models::two_factor::TwoFactorAuth,
        code: &str,
        client_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<bool, AppError> {
        // Check 2FA-specific rate limit
        if self.repo.check_rate_limit(user_id).await? {
            return Err(AppError::BadRequest(
                "Too many failed attempts. Please try again later.".to_string(),
            ));
        }

        let encryption_key = self
            .config
            .two_factor
            .parse_encryption_key()
            .map_err(AppError::BadRequest)?;

        let encrypted_secret = two_factor_data.encrypted_secret.clone();
        let encryption_nonce = two_factor_data.encryption_nonce.clone();
        let code_owned = code.to_string();

        let totp_valid = tokio::task::spawn_blocking(move || {
            let secret = PostgresRepository::decrypt_secret(
                &encrypted_secret,
                &encryption_nonce,
                &encryption_key,
            )?;
            PostgresRepository::verify_totp_code(&secret, &code_owned)
        })
        .await
        .map_err(|e| AppError::BadRequest(format!("Task join error: {}", e)))??;

        let backup_valid = if !totp_valid {
            self.repo.verify_backup_code(user_id, code).await?
        } else {
            false
        };

        if !totp_valid && !backup_valid {
            self.repo.record_failed_attempt(user_id).await?;
            let _ = self
                .repo
                .create_security_audit_log(
                    Some(user_id),
                    audit_events::LOGIN_FAILED,
                    false,
                    client_ip,
                    user_agent,
                    Some(serde_json::json!({"reason": "invalid_2fa_code"})),
                )
                .await;
            return Err(AppError::BadRequest(
                "Invalid two-factor authentication code.".to_string(),
            ));
        }

        self.repo.reset_rate_limit(user_id).await?;
        Ok(backup_valid)
    }
}
