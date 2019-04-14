imports!();

use crate::client::GetQueryBuilder;

// Create a new struct Login with client etc
new_type!(
    Login
    LoginEncryptionKey
);

// Create a from implementation for
from!(
    @GetQueryBuilder
       -> Login = "session"
    @Login
       -> LoginEncryptionKey = "encryptionKey"
);

impl_macro!(
    @Login
       |=> encryption_key -> LoginEncryptionKey
       |
);

exec!(Login);
exec!(LoginEncryptionKey);
