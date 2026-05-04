use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Admin,
    Manager,
    BasicUser,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
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

#[derive(Debug)]
pub enum UserError {
    InvalidEmail,
    PasswordTooShort,
    UserAlreadyDeactivated,
    InsufficientPermissions,
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserError::InvalidEmail => write!(f, "The email address provided is invalid."),
            UserError::PasswordTooShort => write!(f, "Password must be at least 8 characters."),
            UserError::UserAlreadyDeactivated => write!(f, "This user is already deactivated."),
            UserError::InsufficientPermissions => write!(f, "User lacks the required role."),
        }
    }
}
impl std::error::Error for UserError {}

impl User {
    pub fn new(id: String, username: String, password: String, email: EmailAddress) -> Self {
        Self {
            id,
            username,
            password,
            email,
            roles: vec![Role::BasicUser],
            is_active: true,
        }
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