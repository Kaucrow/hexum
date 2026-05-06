use strum::{Display, EnumString};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString)]
pub enum Role {
    Admin,
    Manager,
    BasicUser,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub passwd: String,
    pub email: EmailAddress,
    pub roles: Vec<Role>,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress(String);

impl EmailAddress {
    // Create an EmailAddress.
    // If it doesn't have an '@', it refuses to be created.
    pub fn new(email: String) -> Result<Self, UserError> {
        if !email.contains('@') {
            return Err(UserError::InvalidEmail);
        }
        Ok(Self(email))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("The email address provided is invalid.")]
    InvalidEmail,
    #[error("Password must be at least 8 characters.")]
    PasswordTooShort,
    #[error("This user is already deactivated.")]
    UserAlreadyDeactivated,
    #[error("User lacks the required role.")]
    InsufficientPermissions,
}

impl User {
    pub fn new(username: &String, passwd: &String, email: &String) -> Result<Self, UserError> {
        Ok(Self {
            id: uuid::Uuid::new_v4(),
            username: username.clone(),
            passwd: passwd.clone(),
            email: EmailAddress::new(email.to_string())?,
            roles: vec![Role::BasicUser],
            is_active: true,
        })
    }

    // Check if the user has any of the roles provided
    pub fn has_any_role(&self, allowed_roles: &[Role]) -> bool {
        self.roles.iter().any(|user_role| allowed_roles.contains(user_role))
    }

    // Deactivate a user
    pub fn deactivate(&mut self) -> Result<(), UserError> {
        if !self.is_active {
            return Err(UserError::UserAlreadyDeactivated);
        }

        self.is_active = false;
        Ok(())
    }

    // Give admin permissions to a user
    pub fn grant_admin(&mut self) {
        if !self.roles.contains(&Role::Admin) {
            self.roles.push(Role::Admin);
        }
    }
}